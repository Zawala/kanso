// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appName => 'Kanso';

  @override
  String get libraryAllNotes => 'All Notes';

  @override
  String get libraryTemplates => 'Templates';

  @override
  String get libraryPinned => 'Pinned';

  @override
  String get libraryRecent => 'Recent';

  @override
  String get libraryNotebooks => 'Notebooks';

  @override
  String get libraryTags => 'Tags';

  @override
  String get libraryTrash => 'Trash';

  @override
  String get openSettings => 'Settings';

  @override
  String get openLibrary => 'Library';

  @override
  String get searchHint => 'Search notes';

  @override
  String get searchClear => 'Clear search';

  @override
  String get searchCancel => 'Cancel';

  @override
  String get searchFilterAll => 'All';

  @override
  String get searchFilterTitles => 'Titles';

  @override
  String get searchFilterTags => 'Tags';

  @override
  String get searchFilterNotebook => 'This notebook';

  @override
  String searchResultCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count results',
      one: '1 result',
    );
    return '$_temp0';
  }

  @override
  String noteListCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count notes',
      one: '1 note',
    );
    return '$_temp0';
  }

  @override
  String get noteListMore => 'More';

  @override
  String get noteBack => 'Back';

  @override
  String get noteForward => 'Forward';

  @override
  String get noteSearch => 'Search';

  @override
  String get notePreview => 'Preview';

  @override
  String get noteEdit => 'Edit';

  @override
  String get noteInfo => 'Note info';

  @override
  String get noteMetaCreated => 'Created';

  @override
  String get noteMetaUpdated => 'Updated';

  @override
  String get statusActive => 'Active';

  @override
  String get statusOnHold => 'On hold';

  @override
  String get statusCompleted => 'Completed';

  @override
  String get statusDropped => 'Dropped';

  @override
  String editProperties(int count) {
    return 'Properties · $count';
  }

  @override
  String get accessoryHeading => 'Heading';

  @override
  String get accessoryList => 'List';

  @override
  String get accessoryTask => 'Task';

  @override
  String get accessoryBold => 'Bold';

  @override
  String get accessoryLink => 'Link';

  @override
  String get accessoryImage => 'Image';

  @override
  String get accessoryWikiLink => 'Link to a note';

  @override
  String get accessoryIndent => 'Indent';

  @override
  String get accessoryOutdent => 'Outdent';

  @override
  String get accessoryUndo => 'Undo';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsClose => 'Close';

  @override
  String get settingsAppearance => 'Appearance';

  @override
  String get settingsTheme => 'Theme';

  @override
  String get settingsTextSize => 'Text size';

  @override
  String get settingsSync => 'Sync';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsVersion => 'Version';

  @override
  String get settingsWebsite => 'Website';

  @override
  String get settingsLicences => 'Licences';

  @override
  String get themesTitle => 'Themes';

  @override
  String get themesInstalled => 'Installed';

  @override
  String get themesBadgeBuiltIn => 'Built in';

  @override
  String get themesBadgeInstalled => 'Installed';

  @override
  String get themesCurrent => 'In use';

  @override
  String get lockedTitle => 'This note is locked';

  @override
  String get errorSignedOutTitle => 'Signed out on this device';

  @override
  String get errorSignedOutMessage =>
      'Your session on this device ended. Sign in again to keep syncing; your notes stay here.';

  @override
  String get errorSignInAgain => 'Sign in again';

  @override
  String get errorNotNow => 'Not now';

  @override
  String get designGalleryTitle => 'Design proposal';

  @override
  String get designGalleryIntro =>
      'Static screens for review. Nothing here saves or syncs.';

  @override
  String get designGalleryPhone => 'Phone';

  @override
  String get designGalleryTablet => 'Tablet';

  @override
  String get appearanceSystem => 'System';

  @override
  String get appearanceLight => 'Light';

  @override
  String get appearanceDark => 'Dark';

  @override
  String get screenNoteEdit => 'Note, edit';

  @override
  String get screenError => 'Error card';

  @override
  String get themesSampleHeading => 'Heading';

  @override
  String get themesSampleLine => 'Some **bold** and *italic*';

  @override
  String get themesSampleCode => '`code`, [a link](url)';

  @override
  String get themesSampleQuote => 'a quote';

  @override
  String get settingsFonts => 'Fonts';

  @override
  String get settingsBack => 'Back';

  @override
  String get fontsTitle => 'Fonts';

  @override
  String get fontsEditorSection => 'Editor font';

  @override
  String get fontsEditorFooter => 'The font you write in.';

  @override
  String get fontsPreviewSection => 'Preview font';

  @override
  String get fontsPreviewFooter => 'The font a note is read in.';

  @override
  String get fontsCodeSection => 'Code font';

  @override
  String get fontsCodeFooter => 'Used for code in notes and in theme previews.';

  @override
  String get fontsSystemName => 'System — your device’s font';

  @override
  String get fontsSystemShort => 'System';

  @override
  String get fontsSystemSample => 'San Francisco on iOS, Roboto on Android';

  @override
  String get fontsSampleText => 'Calm notes, clearly written';

  @override
  String get fontsSampleCode => 'let note = merge(a, b);';

  @override
  String get fontsTextSizeSection => 'Text size';

  @override
  String get fontsTextSizeFooter =>
      'Changes the text of notes only. The rest of Kanso follows your device’s text size.';

  @override
  String get fontsTextSizeSample =>
      'Calm software says less and remembers more. Every device keeps a full copy of your notes, and a change is merged on the others.';

  @override
  String get textSizeSmall => 'Small';

  @override
  String get textSizeDefault => 'Default';

  @override
  String get textSizeLarge => 'Large';

  @override
  String get textSizeLarger => 'Larger';

  @override
  String get noteModeLabel => 'View mode';

  @override
  String get noteListLoading => 'Loading notes';

  @override
  String emptyNotebookTitle(String notebook) {
    return 'No notes in $notebook yet';
  }

  @override
  String get emptyNotebookMessage =>
      'Notes added here will appear in this list.';

  @override
  String get noNoteSelectedTitle => 'No note selected';

  @override
  String get noNoteSelectedMessage =>
      'Choose a note from the list to read it here.';

  @override
  String get libraryDesignGallery => 'Design gallery';

  @override
  String get retry => 'Try again';

  @override
  String get libraryLoading => 'Loading the library';

  @override
  String get libraryFailedTitle => 'The library didn\'t load';

  @override
  String get libraryFailedMessage =>
      'Something went wrong reading the notebooks and tags on this device. Try again.';

  @override
  String get libraryNoNotebooks => 'No notebooks yet';

  @override
  String get libraryNoTags => 'No tags yet';

  @override
  String get dayToday => 'Today';

  @override
  String get dayYesterday => 'Yesterday';

  @override
  String get noteUntitled => 'Untitled';

  @override
  String get noteListLocked => 'Locked';

  @override
  String get noteListFailedTitle => 'These notes didn\'t load';

  @override
  String get noteListFailedMessage =>
      'Something went wrong reading the notes on this device. Try again.';

  @override
  String get emptyAllTitle => 'No notes yet';

  @override
  String get emptyAllMessage => 'Notes will appear here.';

  @override
  String get emptyPinnedTitle => 'Nothing pinned';

  @override
  String get emptyPinnedMessage =>
      'Pin a note by swiping it, and it will wait here.';

  @override
  String get emptyRecentTitle => 'Nothing recent';

  @override
  String get emptyRecentMessage =>
      'Notes you have changed lately will appear here.';

  @override
  String get emptyTrashTitle => 'The trash is empty';

  @override
  String get emptyTrashMessage =>
      'Notes you trash wait here until you move them out.';

  @override
  String emptyTagTitle(String tag) {
    return 'No notes tagged $tag';
  }

  @override
  String get emptyTagMessage => 'Notes with this tag will appear in this list.';

  @override
  String get emptyNarrowedTitle => 'No notes match';

  @override
  String get emptyNarrowedMessage =>
      'Try another status, or turn off Pinned Only.';

  @override
  String get listOptionsTitle => 'View options';

  @override
  String get listOptionsSortBy => 'Sort by';

  @override
  String get listOptionsDateUpdated => 'Date Updated';

  @override
  String get listOptionsDateCreated => 'Date Created';

  @override
  String get listOptionsTitleSort => 'Title';

  @override
  String get listOptionsNewestFirst => 'Newest First';

  @override
  String get listOptionsOldestFirst => 'Oldest First';

  @override
  String get listOptionsAToZ => 'A to Z';

  @override
  String get listOptionsZToA => 'Z to A';

  @override
  String get listOptionsStatus => 'Status';

  @override
  String get listOptionsAnyStatus => 'Any Status';

  @override
  String get listOptionsPinnedOnly => 'Pinned Only';

  @override
  String get noteListDone => 'Done';

  @override
  String noteListSelected(int count) {
    return '$count selected';
  }

  @override
  String get noteListPin => 'Pin';

  @override
  String get noteListUnpin => 'Unpin';

  @override
  String get noteListMove => 'Move';

  @override
  String get noteListTrash => 'Trash';

  @override
  String moveSheetTitle(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Move $count notes to',
      one: 'Move 1 note to',
    );
    return '$_temp0';
  }

  @override
  String get noteListCancel => 'Cancel';

  @override
  String get noteListChangeFailed => 'That didn\'t work. Try again.';

  @override
  String get noteListChangeLocked =>
      'Locked notebooks can\'t take or give up notes here yet.';

  @override
  String get noteListDismiss => 'OK';

  @override
  String get noteLoading => 'Loading the note';

  @override
  String get noteNotFoundTitle => 'This note isn\'t here';

  @override
  String get noteNotFoundMessage =>
      'It may have been deleted, or it hasn\'t reached this device yet.';

  @override
  String get noteFailedTitle => 'This note didn\'t load';

  @override
  String get noteFailedMessage =>
      'Something went wrong reading the note on this device. Try again.';

  @override
  String get lockedMessageSealed =>
      'Its text stays sealed. Only its title can be shown here.';

  @override
  String get menuCut => 'Cut';

  @override
  String get menuCopy => 'Copy';

  @override
  String get menuPaste => 'Paste';

  @override
  String get menuSelectAll => 'Select All';

  @override
  String get searchIdleTitle => 'Search your notes';

  @override
  String get searchIdleMessage => 'Find words in titles and text, or a tag.';

  @override
  String get searchNoResultsTitle => 'No notes match';

  @override
  String searchNoResultsMessage(String query) {
    return 'Nothing matched “$query”. Try fewer or different words.';
  }

  @override
  String get searchFailedTitle => 'The search didn\'t run';

  @override
  String get searchFailedMessage =>
      'Something went wrong searching the notes on this device. Try again.';

  @override
  String get searchLoading => 'Searching';

  @override
  String settingsThemeBoth(String light, String dark) {
    return '$light, $dark';
  }

  @override
  String get settingsSyncStatus => 'Status';

  @override
  String get settingsSyncOff => 'Off';

  @override
  String get settingsSyncOn => 'On';

  @override
  String get settingsSyncSyncing => 'Syncing…';

  @override
  String get settingsSyncSynced => 'Synced';

  @override
  String get settingsSyncAttention => 'Needs attention';

  @override
  String get settingsSyncBackupFooter =>
      'The notes on this device aren’t included in device backups.';

  @override
  String get settingsSyncSignIn => 'Sign in';

  @override
  String get settingsSyncOpenNotes => 'Open your notes on this device';

  @override
  String get settingsSyncNow => 'Sync now';

  @override
  String get settingsSyncAccount => 'Account';

  @override
  String get accountCancel => 'Cancel';

  @override
  String get accountDone => 'Done';

  @override
  String get accountRetry => 'Try again';

  @override
  String get accountCopy => 'Copy';

  @override
  String get accountCopied => 'Copied';

  @override
  String get accountSomethingWrong =>
      'Something went wrong. Try again in a moment.';

  @override
  String get accountSignedOutError =>
      'This device was signed out. Sign in again.';

  @override
  String get accountLoading => 'Loading';

  @override
  String get accountUsername => 'Username';

  @override
  String get accountPassword => 'Password';

  @override
  String get accountRepeatPassword => 'Repeat password';

  @override
  String get accountNewPassword => 'New password';

  @override
  String get accountCurrentPassword => 'Current password';

  @override
  String get formProblemUsernameInvalid =>
      'Use 3–32 lowercase letters, digits, - or _, starting and ending with a letter or digit.';

  @override
  String get formProblemUsernameReserved =>
      'That username is reserved. Try another.';

  @override
  String get formProblemPasswordTooShort =>
      'Use at least 12 characters. A few random words work well.';

  @override
  String get formProblemPasswordTooLong => 'Use at most 256 characters.';

  @override
  String get formProblemPasswordSameAsUsername =>
      'Choose a password that isn’t your username.';

  @override
  String get formProblemPasswordsDiffer => 'The passwords don’t match.';

  @override
  String get formProblemCredentialsDontMatch =>
      'That username and password don’t match.';

  @override
  String get formProblemRecoveryKeyMissing =>
      'A recovery key has 28 letters and digits, in groups of four.';

  @override
  String get accountServerLabel => 'Server';

  @override
  String get accountServerDefault => 'Kanso sync';

  @override
  String get accountServerChange => 'Use another server';

  @override
  String get accountServerAddress => 'Server address';

  @override
  String get accountServerAddressHint => 'https://sync.example.com';

  @override
  String get accountServerUse => 'Use this server';

  @override
  String get accountServerUseDefault => 'Use Kanso sync';

  @override
  String get accountServerChecking => 'Checking the server…';

  @override
  String get accountServerNoSignups => 'This server isn’t taking new accounts.';

  @override
  String get accountServerNoMethods =>
      'This server offers no way to sign in that this app can use.';

  @override
  String get accountSignInTitle => 'Sign in';

  @override
  String get accountSignInHeading => 'Sign in to Kanso';

  @override
  String get accountContinueWithApple => 'Continue with Apple';

  @override
  String get accountContinueWithGoogle => 'Continue with Google';

  @override
  String get accountContinueWithGithub => 'Continue with GitHub';

  @override
  String get accountSignInWithPasskey => 'Sign in with a passkey';

  @override
  String get accountOr => 'or';

  @override
  String get accountSignInButton => 'Sign in';

  @override
  String get accountCreateAccountLink => 'Create an account';

  @override
  String get accountForgotPassword =>
      'Forgot your password? Use your recovery key';

  @override
  String get accountSigningIn => 'Signing in…';

  @override
  String get accountCreateTitle => 'Create your account';

  @override
  String get accountCreateButton => 'Create account';

  @override
  String get accountCreatingButton => 'Creating…';

  @override
  String get accountCreateWithPasskey => 'Create with a passkey';

  @override
  String get accountCreateRecoveryNote =>
      'Kanso can’t reset your password; you’ll get a recovery key instead.';

  @override
  String get accountSignInInstead => 'I already have an account';

  @override
  String get accountGithubTitle => 'Continue with GitHub';

  @override
  String get accountGithubExplain => 'Type this code at GitHub to sign in.';

  @override
  String get accountGithubOpen => 'Open GitHub';

  @override
  String get accountGithubWaiting => 'Waiting for you to finish at GitHub…';

  @override
  String get accountProviderCancel => 'Cancel';

  @override
  String get accountProviderWaiting =>
      'Finish signing in in the page that opened. Kanso opens again when you’re done.';

  @override
  String get accountUsernameTitle => 'Choose a username';

  @override
  String get accountUsernameExplain =>
      'Your account is made with this name. You can change it before creating the account.';

  @override
  String get accountRecoveryKeyTitle => 'Your recovery key';

  @override
  String get accountRecoveryKeyText =>
      'If you forget your password, this key and your username are the only way back into your account. Kanso and the server’s owner can’t reset it. Anyone with it can take over your account and read your notes, so keep it somewhere safe, like a password manager or on paper. Each time it’s used, Kanso gives you a new one.';

  @override
  String get accountRecoveryKeyCopy => 'Copy recovery key';

  @override
  String get accountRecoveryKeyConfirm =>
      'I’ve saved my recovery key somewhere safe';

  @override
  String get accountRecoveryKeyNoneTitle => 'Nothing to save';

  @override
  String get accountRecoveryKeyNoneMessage =>
      'There’s no recovery key waiting to be saved.';

  @override
  String get accountRecoveryKeyFailedTitle => 'The recovery key didn’t load';

  @override
  String get accountRecoveryKeyFailedMessage =>
      'Something went wrong reading the recovery key. Try again.';

  @override
  String get bindingAdd => 'Add them to the account';

  @override
  String get bindingAddNote =>
      'They’re encrypted and uploaded. A note deleted in the account but still on this device comes back, and a note changed on both sides keeps both versions.';

  @override
  String get bindingReplace => 'Replace them with the account’s notes';

  @override
  String get bindingReplaceNote =>
      'Kanso saves a backup of this device’s library first, then downloads the account’s notes.';

  @override
  String get bindingCancel => 'Cancel';

  @override
  String get accountUnlockTitle => 'Open your notes on this device';

  @override
  String accountUnlockExplain(String username) {
    return 'You’re signed in as $username, but this device can’t read your notes yet.';
  }

  @override
  String get accountUnlockButton => 'Unlock';

  @override
  String get accountApproveFromDevice => 'Approve from another device';

  @override
  String get accountUseRecoveryKey => 'Use your recovery key';

  @override
  String get accountSignOut => 'Sign out';

  @override
  String get accountApprovalTitle => 'Approve this device';

  @override
  String get accountApprovalIntro =>
      'Open Kanso on a device that’s already signed in and approve this one.';

  @override
  String get accountApprovalRequesting => 'Asking another device…';

  @override
  String get accountApprovalWaiting => 'Waiting for another device…';

  @override
  String get accountApprovalUnlocking => 'Approved. Unlocking this device…';

  @override
  String get accountApprovalApproved => 'This device is unlocked.';

  @override
  String get accountApprovalNoDevices =>
      'No other device answered. Use your recovery key to open this device.';

  @override
  String get accountApprovalCancelled => 'That request was cancelled.';

  @override
  String get accountApprovalExpired => 'That request expired.';

  @override
  String get accountApprovalFailed =>
      'Can’t reach the server. Your notes are safe on this device.';

  @override
  String get accountApprovalCompare =>
      'On your other device, approve the request only if it shows this code.';

  @override
  String get accountApprovalRecoveryInstead => 'Use your recovery key instead';

  @override
  String accountApprovalCode(String code) {
    return 'Code $code';
  }

  @override
  String get accountRecoverTitle => 'Reset your password';

  @override
  String get accountRecoverExplain =>
      'Enter your recovery key and choose a new password. Your other devices sign out and ask for it, and you get a new recovery key to save.';

  @override
  String get accountRecoveryKeyField => 'Recovery key';

  @override
  String get accountRecoveryKeyHint => 'The key you saved, in groups of four.';

  @override
  String get accountRecoverButton => 'Reset password';

  @override
  String get accountRecoveringButton => 'Resetting…';

  @override
  String get accountTitle => 'Account';

  @override
  String accountSignedInAs(String username) {
    return 'Signed in as $username';
  }

  @override
  String get accountLoadFailedTitle => 'Your account didn’t load';

  @override
  String get accountLoadFailedMessage =>
      'Something went wrong reading your account. Try again.';

  @override
  String get accountRecoveryKeyUnsavedTitle =>
      'Your new recovery key isn’t saved yet';

  @override
  String get accountRecoveryKeyUnsavedMessage =>
      'Save it now: the one you had before no longer works.';

  @override
  String get accountRecoveryKeyShow => 'Show it';

  @override
  String get accountDevices => 'Devices';

  @override
  String get accountThisDevice => 'This device';

  @override
  String accountLastSeen(String date) {
    return 'Last seen $date';
  }

  @override
  String get accountDeviceSignOut => 'Sign out';

  @override
  String get accountWaysToSignIn => 'Ways to sign in';

  @override
  String get accountPasswordRow => 'Password';

  @override
  String get accountPasswordChange => 'Change';

  @override
  String get accountPasswordNotSet => 'Not set';

  @override
  String get accountPasskeys => 'Passkeys';

  @override
  String get accountPasskeyAdd => 'Add a passkey';

  @override
  String get accountPasskeyRemove => 'Remove';

  @override
  String accountPasskeyAdded(String date) {
    return 'Added $date';
  }

  @override
  String accountPasskeyUsed(String added, String used) {
    return 'Added $added · Last used $used';
  }

  @override
  String get accountPasskeyCanUnlock => 'Can unlock a new device';

  @override
  String get accountPasskeySignsInOnly => 'Signs in only';

  @override
  String get accountProviderApple => 'Apple';

  @override
  String get accountProviderGoogle => 'Google';

  @override
  String get accountProviderGithub => 'GitHub';

  @override
  String get accountProviderLinked => 'Linked';

  @override
  String get accountProviderNotLinked => 'Not linked';

  @override
  String get accountProviderLink => 'Link';

  @override
  String get accountProviderUnlink => 'Unlink';

  @override
  String get accountNoProofHere =>
      'Confirm it’s you with your password. This device can’t use another way for this account yet.';

  @override
  String get accountRecoveryKeySection => 'Recovery key';

  @override
  String get accountRecoveryKeyNew => 'Make a new one';

  @override
  String get accountRecoveryKeyFooter =>
      'Your recovery key still works if you lose your other ways in.';

  @override
  String get accountStorage => 'Storage';

  @override
  String get accountDeleteAccount => 'Delete account';

  @override
  String get accountWorking => 'Working…';

  @override
  String accountRevokeTitle(String name) {
    return 'Sign out $name?';
  }

  @override
  String get accountRevokeMessage =>
      'It stops syncing until someone signs in on it again. Its notes stay on it.';

  @override
  String accountSignOutTitle(String username) {
    return 'Sign out of $username?';
  }

  @override
  String get accountSignOutMessage =>
      'Notes stay on this device. Sync stops until you sign in again.';

  @override
  String get accountSignOutUnsavedKey =>
      'Your new recovery key hasn’t been saved yet; signing out deletes it.';

  @override
  String get accountChangePasswordTitle => 'Change password';

  @override
  String get accountChangePasswordMessage =>
      'Your other devices sign out and ask for the new password. Your recovery key changes too: the one you saved stops working, and Kanso shows the new one to save.';

  @override
  String get accountNewRecoveryKeyTitle => 'New recovery key';

  @override
  String get accountNewRecoveryKeyMessage =>
      'The key you saved stops working, and Kanso shows the new one to save instead.';

  @override
  String get accountNewRecoveryKeyButton => 'Make a new key';

  @override
  String get accountDeleteTitle => 'Delete account';

  @override
  String accountDeleteMessage(String username) {
    return 'This deletes $username’s encrypted notes, attachments and devices from the server. Notes on this device stay here, and other devices keep their copies. The server’s backups keep a copy for up to 30 days. This can’t be undone.';
  }

  @override
  String accountDeleteTypeUsername(String username) {
    return 'Type $username to confirm';
  }

  @override
  String get accountAddPasskeyTitle => 'Add a passkey';

  @override
  String get accountAddPasskeyMessage =>
      'Confirm it’s you with your password, then use your phone’s passkey sheet.';

  @override
  String accountRemovePasskeyTitle(String label) {
    return 'Remove $label?';
  }

  @override
  String get accountRemovePasskeyMessage =>
      'You won’t be able to sign in with it any more. Confirm it’s you with your password.';

  @override
  String get accountRemovePasskeyButton => 'Remove passkey';

  @override
  String accountLinkTitle(String provider) {
    return 'Link $provider';
  }

  @override
  String get accountLinkMessage =>
      'Confirm it’s you with your password, then finish in the page that opens.';

  @override
  String accountUnlinkTitle(String provider) {
    return 'Unlink $provider?';
  }

  @override
  String get accountUnlinkMessage =>
      'It no longer signs in to this account. Confirm it’s you with your password.';

  @override
  String get accountConfirmSignOut => 'Sign out';

  @override
  String accountNoticeDeviceSignedOut(String name) {
    return 'Signed out $name.';
  }

  @override
  String get accountNoticePasskeyAddedCanUnlock =>
      'Added the passkey. It can unlock: on a new device it signs in and opens your notes.';

  @override
  String get accountNoticePasskeyAddedSignsIn =>
      'Added the passkey. It signs in only: a new device will also need approval or your recovery key.';

  @override
  String get accountNoticePasskeyRemoved => 'Removed the passkey.';

  @override
  String get accountNoticeProviderLinked =>
      'Linked. You can sign in with it now.';

  @override
  String get accountNoticeProviderUnlinked =>
      'Unlinked. It no longer signs in to this account.';

  @override
  String get approveDeviceTitle => 'Approve a device';

  @override
  String approveDeviceMessage(String account, String device, String time) {
    return 'A device wants to join $account: $device, asked at $time.';
  }

  @override
  String get approveDeviceUnnamed => 'A device';

  @override
  String approveDeviceCompare(String code) {
    return 'Does it show $code? Only approve if it does.';
  }

  @override
  String get approveDeviceChanged =>
      'This request changed after it was made. Don’t approve.';

  @override
  String get approveDeviceApprove => 'Approve';

  @override
  String get approveDeviceDecline => 'Don’t approve';

  @override
  String approveDeviceMore(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count more requests are waiting.',
      one: '1 more request is waiting.',
    );
    return '$_temp0';
  }

  @override
  String get accountServerFailedTitle => 'This server didn’t answer';

  @override
  String get accountDeleteMismatch => 'That isn’t this account’s username.';

  @override
  String settingsVersionValue(String version, String build) {
    return '$version ($build)';
  }

  @override
  String get licencesLoading => 'Loading licences';

  @override
  String get licencesFailedTitle => 'The licences didn’t load';

  @override
  String get licencesFailedMessage =>
      'Something went wrong reading the licences. Try again.';

  @override
  String get licencesEmptyTitle => 'No licences';

  @override
  String get licencesEmptyMessage =>
      'The packages Kanso is made of will be listed here.';

  @override
  String licencesCount(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count licences',
      one: '1 licence',
    );
    return '$_temp0';
  }

  @override
  String get themesFollowSystem =>
      'Switch between a light and a dark theme with the system';

  @override
  String get themesFollowSystemOn =>
      'Showing the light theme in light mode and the dark one in dark mode.';

  @override
  String get themesFollowSystemOff =>
      'Off: the theme stays as picked whatever the system’s mode.';

  @override
  String get themesFailedTitle => 'The themes didn’t load';

  @override
  String get themesFailedMessage =>
      'Something went wrong reading the themes. Try again.';

  @override
  String get themesLoading => 'Loading themes';

  @override
  String get backupNoticeTitle => 'Notes stay on this device';

  @override
  String get backupNoticeMessage =>
      'The notes on this device aren’t included in device backups, so losing or resetting the device takes them with it.';

  @override
  String get backupNoticeOk => 'OK';
}
