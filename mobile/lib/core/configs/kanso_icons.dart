// Kanso — kanso_icons.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The icons Kanso draws: Lucide, the same set the desktop app uses.
///
/// Each value names an SVG under `assets/icons/lucide/`. Draw one with
/// `KansoIcon`, which draws the icon theme chosen in Settings and falls back
/// to this SVG for an icon the theme leaves out. The names are also the list
/// of icons the Rust core resolves a theme for
/// (`crates/kanso-mobile/app-icons.txt`).
enum KansoIcons {
  textSize('a-large-small'),
  archive('archive'),
  arrowLeft('arrow-left'),
  arrowRight('arrow-right'),
  arrowUpDown('arrow-up-down'),
  bell('bell'),
  bold('bold'),
  book('book'),
  bookOpen('book-open'),
  calendar('calendar'),
  camera('camera'),
  caseSensitive('case-sensitive'),
  check('check'),
  chevronDown('chevron-down'),
  chevronLeft('chevron-left'),
  chevronRight('chevron-right'),
  circleCheck('circle-check'),
  circleDot('circle-dot'),
  circlePause('circle-pause'),
  circleHelp('circle-question-mark'),
  circleUser('circle-user'),
  circleX('circle-x'),
  clock('clock'),
  cloud('cloud'),
  codeXml('code-xml'),
  copy('copy'),
  databaseBackup('database-backup'),
  download('download'),
  ellipsis('ellipsis'),
  externalLink('external-link'),
  eye('eye'),
  fileText('file-text'),
  files('files'),
  fingerprint('fingerprint-pattern'),
  github('github'),
  globe('globe'),
  hash('hash'),
  image('image'),
  inbox('inbox'),
  info('info'),
  layoutTemplate('layout-template'),
  link('link'),
  list('list'),
  listChecks('list-checks'),
  indentDecrease('list-indent-decrease'),
  indentIncrease('list-indent-increase'),
  lock('lock'),
  lockOpen('lock-open'),
  logOut('log-out'),
  menu('menu'),
  messageSquare('message-square'),
  monitor('monitor'),
  moon('moon'),
  notebook('notebook'),
  palette('palette'),
  panelLeft('panel-left'),
  pencil('pencil'),
  pin('pin'),
  plus('plus'),
  refreshCw('refresh-cw'),
  regex('regex'),
  replaceAll('replace-all'),
  scale('scale'),
  scanFace('scan-face'),
  search('search'),
  settings('settings'),
  shapes('shapes'),
  share('share-2'),
  shield('shield'),
  shieldAlert('shield-alert'),
  shieldCheck('shield-check'),
  sparkles('sparkles'),
  star('star'),
  sun('sun'),
  tag('tag'),
  trash('trash'),
  typeface('type'),
  undo('undo-2'),
  user('user'),
  users('users'),
  wholeWord('whole-word');

  const KansoIcons(this.lucideName);

  final String lucideName;

  static KansoIcons? fromLucideName(String name) {
    for (final icon in KansoIcons.values) {
      if (icon.lucideName == name) return icon;
    }
    return null;
  }
}
