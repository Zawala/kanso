// Kanso — fixture_shared_notes_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/shared_notes_repository.dart';

import 'sharing_fixtures.dart';

/// Shared notes for widget tests that behave like the real ones, in memory:
/// people are invited and given roles, links are made and cancelled,
/// invitations are accepted and declined, and a locked note is refused. A
/// test sets up who exists and what is shared, and reads [calls] to see what
/// was done.
class FixtureSharedNotesRepository implements SharedNotesRepository {
  /// The notes that are shared, by note id.
  final Map<String, ShareInfo> shares = <String, ShareInfo>{};

  /// The people who can be found by username, and what finding them says.
  final Map<String, PersonLookup> people = <String, PersonLookup>{};

  /// The notes that are locked, and the reasons other notes can't be shared.
  final Set<String> locked = <String>{};
  final Map<String, String> unshareable = <String, String>{};

  /// The invitations waiting.
  List<InvitationRow> invitationList = <InvitationRow>[];

  /// No account is signed in: every call fails with [SignedOutError].
  bool signedOut = false;

  /// When set, every call fails with it.
  ApplicationError? failure;

  /// When set, the next call to [info] waits for it.
  Completer<void>? gate;

  /// What was done, as words: `invite n1 ana editor`.
  final List<String> calls = <String>[];

  /// The notes whose live room is open.
  final Set<String> live = <String>{};

  /// What [join] answers.
  JoinRow joined = const JoinRow(
    shareId: 's_new',
    owner: 'ana',
    role: ShareRole.editor,
  );

  /// The address [createLink] gives.
  String linkAddress = 'https://kansoapp.app/join/k3x9#secret';

  /// What a sync of shared notes reports; tests add to it.
  final StreamController<SharedEvent> eventsController =
      StreamController<SharedEvent>.broadcast();

  ApplicationError? _refusal() {
    if (signedOut) {
      return const ApplicationError.signedOut(message: 'Not signed in.');
    }
    return failure;
  }

  Either<ApplicationError, T> _fail<T>(ApplicationError error) =>
      Left<ApplicationError, T>(error);

  Either<ApplicationError, ShareInfo> _changed(
    String noteId,
    ShareInfo Function(ShareInfo info) change,
  ) {
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    final current = shares[noteId];
    if (current == null) {
      return _fail(const ApplicationError.notFound(message: 'Not shared.'));
    }
    final next = change(current);
    shares[noteId] = next;
    return Right<ApplicationError, ShareInfo>(next);
  }

  ShareInfo _with(
    ShareInfo info, {
    List<MemberRow>? members,
    List<ShareLinkRow>? links,
    List<MemberRow>? requests,
  }) {
    return ShareInfo(
      shareId: info.shareId,
      noteId: info.noteId,
      myRole: info.myRole,
      owner: info.owner,
      members: members ?? info.members,
      links: links ?? info.links,
      requests: requests ?? info.requests,
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> canShare(String noteId) async {
    calls.add('canShare $noteId');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    if (locked.contains(noteId)) {
      return _fail(const ApplicationError.locked(message: 'Locked.'));
    }
    final reason = unshareable[noteId];
    if (reason != null) {
      return _fail(ApplicationError.invalid(message: reason));
    }
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> share(
    String noteId,
    List<(String, ShareRole)> invites,
  ) async {
    calls.add(
      'share $noteId ${invites.map((i) => '${i.$1} ${i.$2.name}').join(', ')}',
    );
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    final info = shareInfo(
      noteId,
      members: <MemberRow>[
        memberRow('ngoni', role: ShareRole.owner, me: true, verified: true),
        for (final (name, role) in invites)
          memberRow(name, role: role, invited: true, verified: true),
      ],
    );
    shares[noteId] = info;
    return Right<ApplicationError, ShareInfo>(info);
  }

  @override
  Future<Either<ApplicationError, ShareInfo?>> info(String noteId) async {
    calls.add('info $noteId');
    await gate?.future;
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    if (locked.contains(noteId)) {
      return _fail(const ApplicationError.locked(message: 'Locked.'));
    }
    return Right<ApplicationError, ShareInfo?>(shares[noteId]);
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> invite(
    String noteId,
    String username,
    ShareRole role,
  ) async {
    calls.add('invite $noteId $username ${role.name}');
    return _changed(
      noteId,
      (info) => _with(
        info,
        members: <MemberRow>[
          ...info.members,
          memberRow(username, role: role, invited: true, verified: true),
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> setRole(
    String noteId,
    String accountId,
    ShareRole role,
  ) async {
    calls.add('setRole $noteId $accountId ${role.name}');
    return _changed(
      noteId,
      (info) => _with(
        info,
        members: <MemberRow>[
          for (final member in info.members)
            if (member.accountId == accountId)
              memberRow(
                member.username,
                role: role,
                fingerprint: member.fingerprint,
                keyChanged: member.keyChanged,
                invited: member.invited,
                verified: member.verified,
              )
            else
              member,
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> remove(
    String noteId,
    String accountId,
  ) async {
    calls.add('remove $noteId $accountId');
    return _changed(
      noteId,
      (info) => _with(
        info,
        members: <MemberRow>[
          for (final member in info.members)
            if (member.accountId != accountId) member,
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> approveRequest(
    String noteId,
    String accountId,
  ) async {
    calls.add('approve $noteId $accountId');
    return _changed(noteId, (info) {
      final asked = info.requests.where((r) => r.accountId == accountId);
      return _with(
        info,
        members: <MemberRow>[...info.members, ...asked],
        requests: <MemberRow>[
          for (final request in info.requests)
            if (request.accountId != accountId) request,
        ],
      );
    });
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> declineRequest(
    String noteId,
    String accountId,
  ) async {
    calls.add('declineRequest $noteId $accountId');
    return _changed(
      noteId,
      (info) => _with(
        info,
        requests: <MemberRow>[
          for (final request in info.requests)
            if (request.accountId != accountId) request,
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, String>> createLink(
    String noteId,
    ShareRole role,
    int days,
  ) async {
    calls.add('createLink $noteId ${role.name} $days');
    final made = _changed(
      noteId,
      (info) => _with(
        info,
        links: <ShareLinkRow>[
          ...info.links,
          ShareLinkRow(
            id: 'l_${info.links.length + 1}',
            role: role,
            expiresMs:
                DateTime(2026, 10, 26, 10, 30).millisecondsSinceEpoch +
                days * 86400000,
          ),
        ],
      ),
    );
    return made.map((_) => linkAddress);
  }

  @override
  Future<Either<ApplicationError, ShareInfo>> cancelLink(
    String noteId,
    String linkId,
  ) async {
    calls.add('cancelLink $noteId $linkId');
    return _changed(
      noteId,
      (info) => _with(
        info,
        links: <ShareLinkRow>[
          for (final link in info.links)
            if (link.id != linkId) link,
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, JoinRow>> join(String link) async {
    calls.add('join $link');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    return Right<ApplicationError, JoinRow>(joined);
  }

  @override
  Future<Either<ApplicationError, Unit>> stop(String noteId) async {
    calls.add('stop $noteId');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    shares.remove(noteId);
    live.remove(noteId);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> leave(String noteId) async {
    calls.add('leave $noteId');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    shares.remove(noteId);
    live.remove(noteId);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, List<InvitationRow>>> invitations() async {
    calls.add('invitations');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    return Right<ApplicationError, List<InvitationRow>>(
      List<InvitationRow>.of(invitationList),
    );
  }

  @override
  Future<Either<ApplicationError, String>> accept(String shareId) async {
    calls.add('accept $shareId');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    invitationList = <InvitationRow>[
      for (final row in invitationList)
        if (row.shareId != shareId) row,
    ];
    return Right<ApplicationError, String>('n_from_$shareId');
  }

  @override
  Future<Either<ApplicationError, Unit>> decline(String shareId) async {
    calls.add('decline $shareId');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    invitationList = <InvitationRow>[
      for (final row in invitationList)
        if (row.shareId != shareId) row,
    ];
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, PersonLookup>> lookupPerson(
    String username,
  ) async {
    calls.add('lookup $username');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    final found = people[username];
    if (found == null) {
      return _fail(
        ApplicationError.invalid(
          message:
              'There’s no Kanso account called $username that can share yet. '
              'They need to sign in to Kanso once first.',
        ),
      );
    }
    return Right<ApplicationError, PersonLookup>(found);
  }

  @override
  Future<Either<ApplicationError, Unit>> verifyPerson(
    String accountId,
  ) async {
    calls.add('verify $accountId');
    for (final entry in people.entries.toList()) {
      final lookup = entry.value;
      if (lookup is PersonLookup_Found &&
          lookup.field0.accountId == accountId) {
        people[entry.key] = PersonLookup.found(
          personRow(
            lookup.field0.username,
            fingerprint: lookup.field0.fingerprint,
            verified: true,
          ),
        );
      }
    }
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, PersonRow>> acceptNewKey(
    String username,
    String fingerprint,
  ) async {
    calls.add('acceptNewKey $username $fingerprint');
    final refused = _refusal();
    if (refused != null) {
      return _fail(refused);
    }
    final person = personRow(username, fingerprint: fingerprint);
    people[username] = PersonLookup.found(person);
    return Right<ApplicationError, PersonRow>(person);
  }

  @override
  Future<Either<ApplicationError, SharedEvent>> syncShared() async {
    calls.add('syncShared');
    return const Right<ApplicationError, SharedEvent>(
      SharedEvent(
        noteIds: <String>[],
        invitations: 0,
        commentNotes: <String>[],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> openLive(
    String noteId,
    Object holder,
  ) async {
    calls.add('openLive $noteId');
    live.add(noteId);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> closeLive(
    String noteId,
    Object holder,
  ) async {
    calls.add('closeLive $noteId');
    live.remove(noteId);
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Stream<SharedEvent> events() => eventsController.stream;
}
