// Kanso — backup_exclusion_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:xml/xml.dart';

/// The Android resources the test reads, from `mobile/`.
const String _main = 'android/app/src/main';

/// Every domain Android backs up from, in both old and new rules.
const Set<String> _domains = <String>{
  'root',
  'file',
  'database',
  'sharedpref',
  'external',
  'device_root',
  'device_file',
  'device_database',
  'device_sharedpref',
};

const String _androidNamespace = 'http://schemas.android.com/apk/res/android';

XmlDocument _parse(String path) =>
    XmlDocument.parse(File('$_main/$path').readAsStringSync());

String? _attribute(XmlElement element, String name) =>
    element.getAttribute(name, namespaceUri: _androidNamespace);

/// The domains [section] excludes, each of them whole (`path="."`).
Set<String> _excludedDomains(XmlElement section) {
  final domains = <String>{};
  for (final exclude in section.findElements('exclude')) {
    expect(exclude.getAttribute('path'), '.', reason: 'excludes only a part');
    domains.add(exclude.getAttribute('domain')!);
  }
  return domains;
}

void main() {
  group('Android keeps the app out of backups and transfers', () {
    final application = _parse(
      'AndroidManifest.xml',
    ).rootElement.getElement('application')!;

    test('the manifest turns backup off and names both rule files', () {
      expect(_attribute(application, 'allowBackup'), 'false');
      expect(_attribute(application, 'fullBackupContent'), '@xml/backup_rules');
      expect(
        _attribute(application, 'dataExtractionRules'),
        '@xml/data_extraction_rules',
      );
    });

    test('the rule files the manifest names are there', () {
      expect(File('$_main/res/xml/backup_rules.xml').existsSync(), isTrue);
      expect(
        File('$_main/res/xml/data_extraction_rules.xml').existsSync(),
        isTrue,
      );
    });

    test('Android 6 to 11: every domain is excluded, whole', () {
      final root = _parse('res/xml/backup_rules.xml').rootElement;
      expect(root.name.local, 'full-backup-content');
      expect(root.findElements('include'), isEmpty);
      expect(_excludedDomains(root), _domains);
    });

    test('Android 12 and later: cloud backup excludes every domain', () {
      final root = _parse('res/xml/data_extraction_rules.xml').rootElement;
      expect(root.name.local, 'data-extraction-rules');
      final section = root.getElement('cloud-backup')!;
      expect(section.findElements('include'), isEmpty);
      expect(_excludedDomains(section), _domains);
    });

    test('Android 12 and later: a device transfer excludes every domain', () {
      final root = _parse('res/xml/data_extraction_rules.xml').rootElement;
      final section = root.getElement('device-transfer')!;
      expect(section.findElements('include'), isEmpty);
      expect(_excludedDomains(section), _domains);
    });

    test('Android 12 and later: there is no other section', () {
      final root = _parse('res/xml/data_extraction_rules.xml').rootElement;
      expect(
        root.childElements.map((element) => element.name.local).toSet(),
        <String>{'cloud-backup', 'device-transfer'},
      );
    });
  });
}
