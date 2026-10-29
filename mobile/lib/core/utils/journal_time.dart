// Kanso — journal_time.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart';

/// The wall-clock reading of [time] in the device's zone, as the library
/// takes a moment.
LocalTime localTimeOf(DateTime time) {
  final local = time.toLocal();
  return LocalTime(
    year: local.year,
    month: local.month,
    day: local.day,
    hour: local.hour,
    minute: local.minute,
    second: local.second,
  );
}

/// The day [time] falls on in the device's zone.
DayDate dayDateOf(DateTime time) {
  final local = time.toLocal();
  return DayDate(year: local.year, month: local.month, day: local.day);
}

/// Midnight of [day] on the device's clock.
DateTime dateTimeOf(DayDate day) => DateTime(day.year, day.month, day.day);

/// The first column of a calendar whose weeks start on the weekday
/// [firstDayOfWeekIndex] counts from Sunday (0) through Saturday (6): 0 for
/// Monday through 6 for Sunday, as the library counts.
int weekStartOf(int firstDayOfWeekIndex) => (firstDayOfWeekIndex + 6) % 7;

/// Which weekday a calendar starts its weeks on in [region], counted from
/// Sunday (0) through Saturday (6).
///
/// The regions that start on Sunday or Saturday are listed; every other
/// region starts on Monday, which is the rule in most of the world. A null
/// or unknown [region] starts on Monday.
int firstDayOfWeekIndexFor(String? region) {
  final code = region?.toUpperCase();
  if (code == null || code.isEmpty) {
    return 1;
  }
  if (_sundayStart.contains(code)) {
    return 0;
  }
  if (_saturdayStart.contains(code)) {
    return 6;
  }
  return 1;
}

/// Regions whose weeks begin on Sunday.
const Set<String> _sundayStart = <String>{
  'AG',
  'AS',
  'BD',
  'BR',
  'BS',
  'BT',
  'BW',
  'BZ',
  'CA',
  'CN',
  'CO',
  'DM',
  'DO',
  'GT',
  'GU',
  'HK',
  'HN',
  'ID',
  'IL',
  'IN',
  'JM',
  'JP',
  'KH',
  'KR',
  'LA',
  'MH',
  'MM',
  'MO',
  'MT',
  'MX',
  'MZ',
  'NI',
  'NP',
  'PA',
  'PE',
  'PH',
  'PK',
  'PR',
  'PY',
  'SG',
  'SV',
  'TH',
  'TT',
  'TW',
  'UM',
  'US',
  'VE',
  'VI',
  'WS',
  'ZA',
  'ZW',
};

/// Regions whose weeks begin on Saturday, and those that begin on Friday,
/// which is drawn as Saturday's column so the week still has seven days.
const Set<String> _saturdayStart = <String>{
  'AE',
  'AF',
  'BH',
  'DJ',
  'DZ',
  'EG',
  'IQ',
  'IR',
  'JO',
  'KW',
  'LY',
  'OM',
  'QA',
  'SA',
  'SD',
  'SY',
  'YE',
};
