// Kanso — crash_report_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import '../models/data/crash_report.dart';

/// The exact text of [report]: what the person is shown and what is sent.
String crashReportText(CrashReport report) =>
    const JsonEncoder.withIndent('  ').convert(report.toJson());
