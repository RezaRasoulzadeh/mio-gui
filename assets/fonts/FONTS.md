# Bundled fonts

All bundled families use the SIL Open Font License 1.1. Their license files are stored beside the font binaries.

| Family | File | Guaranteed baseline coverage | SHA-256 |
| --- | --- | --- | --- |
| Vazirmatn | `vazirmatn/Vazirmatn-Regular.ttf` and bundled weights | Persian, Arabic, Latin | See repository history for the original supplied files |
| Noto Sans | `noto-sans/NotoSans-Variable.ttf` | Latin, Greek, Cyrillic | `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d` |
| Noto Sans Devanagari | `noto-sans-devanagari/NotoSansDevanagari-Variable.ttf` | Devanagari | `9ce7b04f60e363d8870e5997744cf85cf69d38a4d7d129d364d92a3b14b461d7` |
| Noto Sans Thai | `noto-sans-thai/NotoSansThai-Variable.ttf` | Thai | `5a1c559bb539583c8a1fd99d1c5b9491e5e14478c9cd2bd0970d5c3096cc9ef8` |

The Noto files were downloaded from the corresponding `ofl` directories in the official Google Fonts repository. The framework removes same-named system faces before loading these files so installed fonts cannot override the bundled versions.
