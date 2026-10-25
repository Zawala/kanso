//! The new device's side of "Unlocking a new device", and the approver's
//! checks (docs/ACCOUNTS.md › Unlocking a new device): where the request is,
//! what the waiting page says, and what the approver does with the request
//! at the front. The windows that show them are the apps'.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use kanso_sync::{
    AccountError, ApprovalKeys, OpenedRequest, PendingApproval, PinnedRequest, Session,
    open_request,
};

use super::words;

/// How long the new device waits before it offers the recovery key.
pub const RECOVERY_AFTER_MS: i64 = 120_000;
/// How often the waiting device asks its request.
pub const POLL_EVERY: Duration = Duration::from_secs(2);

/// Where the new device's request is, for its waiting page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitPhase {
    /// Asking the account's other devices.
    Requesting,
    /// The request is waiting for a device to answer.
    Waiting,
    /// A device answered: show the code and post the sealed name.
    Compare {
        code: String,
    },
    /// The data key is on its way.
    Unlocking,
    /// A device approved: the key arrived.
    Approved,
    /// No other device is signed in; the recovery key is the only way.
    NoDevices,
    Cancelled,
    Expired,
    Failed(String),
}

/// The waiting page's state. `started_ms` is when the request was made.
#[derive(Clone, Debug)]
pub struct WaitingState {
    pub phase: WaitPhase,
    pub code: Option<String>,
    pub started_ms: i64,
    pub other_devices: usize,
    /// False until the create answer says how many other devices could answer.
    pub count_known: bool,
    pub recovery_offered: bool,
    /// The server's request ID, once it exists.
    pub request_id: Option<String>,
    /// Set by the waiting page's Cancel; the polling thread asks the server
    /// to cancel and stops.
    pub cancel: Arc<AtomicBool>,
}

impl WaitingState {
    pub fn new(other_devices: usize, started_ms: i64) -> WaitingState {
        WaitingState {
            phase: WaitPhase::Requesting,
            code: None,
            started_ms,
            other_devices,
            count_known: true,
            recovery_offered: other_devices == 0,
            request_id: None,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    /// The state while the create request is in flight. The count is not
    /// known yet, so recovery is never offered early; `know_devices` settles
    /// it when the create answer arrives.
    pub fn requesting(started_ms: i64) -> WaitingState {
        let mut state = WaitingState::new(0, started_ms);
        state.count_known = false;
        state.recovery_offered = false;
        state
    }

    /// The create answer's count. With no other device the recovery key is
    /// offered at once; otherwise only after two minutes.
    pub fn know_devices(&mut self, other_devices: usize, now_ms: i64) {
        self.other_devices = other_devices;
        self.count_known = true;
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered = other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }

    /// The recovery key is offered at once with no other device, otherwise
    /// after two minutes. Before the count is known, nothing is offered.
    pub fn offer_recovery(&mut self, now_ms: i64) {
        if !self.count_known {
            return;
        }
        let elapsed = now_ms.saturating_sub(self.started_ms);
        self.recovery_offered |= self.other_devices == 0 || elapsed >= RECOVERY_AFTER_MS;
    }
}

/// What the waiting page says above the code.
pub fn waiting_line(state: &WaitingState) -> &'static str {
    match &state.phase {
        WaitPhase::Requesting => "Asking another device…",
        WaitPhase::Waiting => "Waiting for another device…",
        WaitPhase::Compare { .. } => "Waiting for another device…",
        WaitPhase::Unlocking => "Approved. Unlocking this Mac…",
        WaitPhase::Approved => "This Mac is unlocked.",
        WaitPhase::NoDevices => "No other device answered. Use your recovery key to open this Mac.",
        WaitPhase::Cancelled => "That request was cancelled.",
        WaitPhase::Expired => "That request expired.",
        WaitPhase::Failed(_) => words::CANT_REACH,
    }
}

/// How a request's name reads before the sealed name arrives.
pub fn request_title(_pending: &PendingApproval, opened_name: Option<&str>) -> String {
    match opened_name {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => "A device".to_owned(),
    }
}

// ---------------------------------------------------------------------------
// The approver's checks
// ---------------------------------------------------------------------------

/// This device's ephemeral key for one request, with the request as it read
/// when the key was made. The pin is what the revealed key is checked
/// against later, never the row a later GET returns.
pub struct ApproverKey {
    pub keys: ApprovalKeys,
    pub pin: PinnedRequest,
}

impl ApproverKey {
    pub fn new(pending: &PendingApproval) -> ApproverKey {
        ApproverKey {
            keys: ApprovalKeys::generate(),
            pin: PinnedRequest::of(pending),
        }
    }
}

/// What the sheet does next with its front request.
#[derive(Debug)]
pub enum FrontStep {
    /// Post this device's key.
    Challenge,
    /// Nothing to show yet: the requester hasn't revealed its key and name.
    Wait,
    /// The request checks out: show its name and code.
    Open(OpenedRequest),
    /// Don't approve.
    Refuse(AccountError),
}

/// Decides the front request's next step from the latest row and this
/// device's pinned key. `challenged` is whether this device already posted
/// its key for the request.
pub fn front_step(
    session: &Session,
    pending: &PendingApproval,
    key: &ApproverKey,
    challenged: bool,
) -> FrontStep {
    if !key.pin.matches(pending) {
        return FrontStep::Refuse(AccountError::ApprovalChanged);
    }
    if pending.epk.is_none() || pending.sealed_name.is_none() {
        return if pending.approver_epk.is_none() && !challenged {
            FrontStep::Challenge
        } else {
            FrontStep::Wait
        };
    }
    if pending
        .approver_epk
        .is_some_and(|stored| stored != key.keys.epk())
    {
        return FrontStep::Refuse(AccountError::ApprovalChanged);
    }
    match open_request(session, pending, &key.pin, &key.keys) {
        Ok(Some(opened)) => FrontStep::Open(opened),
        Ok(None) => FrontStep::Wait,
        Err(err) => FrontStep::Refuse(err),
    }
}

#[cfg(test)]
mod tests {
    use super::super::words;
    use super::{
        ApproverKey, FrontStep, RECOVERY_AFTER_MS, WaitPhase, WaitingState, front_step,
        waiting_line,
    };
    use kanso_sync::api::LoginDevice;
    use kanso_sync::{
        AccountError, ApprovalKeys, DeviceIdentity, LockedSession, MemorySecrets, MemoryServer,
        Password, RequestState, Session, SessionToken, challenge_request, complete_request,
        finish_approval, finish_unlock, name_device, pending_approvals, poll_approval,
        request_approval, sign_in, sign_up,
    };

    const SERVER: &str = "https://sync.kansoapp.app";

    fn device(name: &str, secret: u8) -> DeviceIdentity {
        DeviceIdentity::from_parts(&format!("l_{:032x}", u64::from(secret)), name, [secret; 32])
    }

    /// Alice's unlocked Mac and a second Mac signed in but locked, which
    /// asks her to approve it.
    fn two_devices() -> (MemoryServer, Session, DeviceIdentity, LockedSession) {
        let server = MemoryServer::new();
        let password = Password::new("correct horse battery staple", "alice").unwrap();
        let alice = sign_up(
            &server,
            &MemorySecrets::default(),
            SERVER,
            "alice",
            &password,
            &device("Alex's MacBook Air", 1),
            &|_| {},
        )
        .unwrap()
        .session;
        let new_device = device("Alex's Mac mini", 2);
        let signed_in = sign_in(&server, SERVER, "alice", &password, &new_device, &|_| {})
            .unwrap()
            .session;
        let token = SessionToken::new(
            signed_in
                .authorization()
                .trim_start_matches("Bearer ")
                .to_owned(),
        );
        let locked = LockedSession::new(
            signed_in.server.clone(),
            signed_in.origin.clone(),
            signed_in.account_id.clone(),
            signed_in.username.clone(),
            signed_in.device_id.clone(),
            token,
            signed_in.key_id(),
            "0000000000000000".to_owned(),
            None,
            None,
        )
        .with_login_device(LoginDevice::for_identity(
            &new_device,
            &signed_in.origin,
            &signed_in.account_id,
        ));
        (server, alice, new_device, locked)
    }

    #[test]
    fn the_sheet_opens_an_honest_request_and_its_approval_unlocks_the_new_device() {
        let (server, alice, new_device, locked) = two_devices();
        let keys_new = ApprovalKeys::generate();
        let request = request_approval(&server, &locked, &keys_new).unwrap();

        // The sheet sees the request, makes and pins its key, and challenges.
        let pending = pending_approvals(&server, &alice).unwrap();
        let key = ApproverKey::new(&pending[0]);
        assert!(matches!(
            front_step(&alice, &pending[0], &key, false),
            FrontStep::Challenge
        ));
        challenge_request(&server, &alice, &pending[0], &key.keys).unwrap();
        let pending = pending_approvals(&server, &alice).unwrap();
        assert!(matches!(
            front_step(&alice, &pending[0], &key, true),
            FrontStep::Wait
        ));

        // The new device reveals its key and names itself.
        let RequestState::Challenged { approver_epk, .. } =
            poll_approval(&server, &locked, &request).unwrap()
        else {
            panic!("challenged");
        };
        let approval_new = complete_request(&locked, &request, &keys_new, &approver_epk).unwrap();
        name_device(&server, &locked, &request, &approval_new, &new_device).unwrap();

        // The sheet opens it: same code, the device's name, and approving
        // unlocks the new device.
        let pending = pending_approvals(&server, &alice).unwrap();
        let FrontStep::Open(opened) = front_step(&alice, &pending[0], &key, true) else {
            panic!("an honest request opens");
        };
        assert_eq!(opened.name, "Alex's Mac mini");
        assert_eq!(opened.approval.code(), approval_new.code());
        finish_approval(&server, &alice, &pending[0], &opened.approval).unwrap();
        let RequestState::Approved(approved) = poll_approval(&server, &locked, &request).unwrap()
        else {
            panic!("approved");
        };
        let unlocked =
            finish_unlock(&server, &locked, &approval_new, &approved, &new_device).unwrap();
        assert_eq!(unlocked.session.key_id(), alice.key_id());
    }

    #[test]
    fn the_sheet_refuses_a_request_whose_commitment_changed_after_it_was_pinned() {
        let (server, alice, _new_device, locked) = two_devices();
        request_approval(&server, &locked, &ApprovalKeys::generate()).unwrap();
        let pending = pending_approvals(&server, &alice).unwrap();
        let key = ApproverKey::new(&pending[0]);
        let mut swapped = pending[0].clone();
        swapped.commit = ApprovalKeys::generate().commitment();
        assert!(matches!(
            front_step(&alice, &swapped, &key, false),
            FrontStep::Refuse(AccountError::ApprovalChanged)
        ));
    }

    #[test]
    fn recovery_is_offered_at_once_with_no_device_and_after_two_minutes() {
        let mut state = WaitingState::new(0, 1_000);
        assert!(state.recovery_offered);
        state.offer_recovery(1_000);
        assert!(state.recovery_offered);

        let mut state = WaitingState::new(1, 1_000);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS - 1);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);
    }

    #[test]
    fn recovery_waits_for_the_create_answer_before_it_can_be_offered() {
        let mut state = WaitingState::requesting(1_000);
        assert!(!state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(!state.recovery_offered);
        state.know_devices(1, 1_000);
        assert!(state.count_known);
        assert!(!state.recovery_offered);
        state.offer_recovery(1_000 + RECOVERY_AFTER_MS);
        assert!(state.recovery_offered);

        let mut state = WaitingState::requesting(1_000);
        state.know_devices(0, 1_000);
        assert!(state.recovery_offered);
    }

    #[test]
    fn the_waiting_page_says_what_is_happening() {
        let mut state = WaitingState::new(1, 0);
        assert_eq!(waiting_line(&state), "Asking another device…");
        state.phase = WaitPhase::Waiting;
        assert_eq!(waiting_line(&state), "Waiting for another device…");
        state.phase = WaitPhase::Compare {
            code: "123 456".into(),
        };
        assert_eq!(waiting_line(&state), "Waiting for another device…");
        state.phase = WaitPhase::NoDevices;
        assert!(waiting_line(&state).contains("recovery key"));
        state.phase = WaitPhase::Failed("x".into());
        assert_eq!(waiting_line(&state), words::CANT_REACH);
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// The recovery fallback depends only on the count and elapsed time.
        #[test]
        fn prop_recovery_offer_follows_time_and_count(
            other in 0usize..=9,
            started in 0i64..2_000_000,
            elapsed in -10_000i64..400_000,
        ) {
            let mut state = WaitingState::new(other, started);
            state.recovery_offered = other == 0;
            state.offer_recovery(started + elapsed);
            let expected = other == 0 || elapsed >= RECOVERY_AFTER_MS;
            proptest::prop_assert_eq!(state.recovery_offered, expected);
        }
    }
}
