<!-- tsrust-docs
{"id":"reference-standard-library-locale","group":"standard-library","order":28,"status":"partial","targets":["native"],"api":[{"receiver":"std:locale","name":"Locale"},{"receiver":"std:locale","name":"Collator"},{"receiver":"std:locale","name":"Segmenter"},{"receiver":"std:locale","name":"NumberFormat"},{"receiver":"std:locale","name":"SegmentRange"},{"receiver":"std:locale","name":"LocaleError"},{"receiver":"std:locale","name":"CollatorOptions"},{"receiver":"std:locale","name":"NumberFormatOptions"},{"receiver":"std:locale","name":"parseLocale"},{"receiver":"std:locale","name":"formatLocale"},{"receiver":"std:locale","name":"createCollator"},{"receiver":"std:locale","name":"createSegmenter"},{"receiver":"std:locale","name":"createNumberFormat"},{"receiver":"Collator","name":"compare"},{"receiver":"Segmenter","name":"segment"},{"receiver":"NumberFormat","name":"format"}]}
-->
# Explicit locales

Use explicit locales to compare strings, segment graphemes and format exact numbers. The API reference below documents locale fallback, supported options and ownership; it does not infer the host user locale.
