global CLX_Lang := CLX_Config("Core", "Language", "auto", "语言切换")
global CLX_i18nConfigDir := "Core/locales/"
global CLX_i18nConfigPath := "Core/lang.ini"
global CLX_i18n_IsCLX_InstalledByGitClone := FileExist(A_WorkingDir "/.git")
global CLX_i18n_newTranslatedSaveTo := "Core/lang.ini"

CONVERT_FILE_TO_UTF16_WITH_BOM_ENCODING(CLX_i18nConfigPath)

; - [Language Codes \| AutoHotkey v1]( https://www.autohotkey.com/docs/v1/misc/Languages.htm )

LCID_0036 := "Afrikaans"  ; af
LCID_0036_CODE := "af"
LCID_0436 := "Afrikaans (South Africa)"  ; af-ZA
LCID_0436_CODE := "af-ZA"
LCID_001C := "Albanian"  ; sq
LCID_001C_CODE := "sq"
LCID_041C := "Albanian (Albania)"  ; sq-AL
LCID_041C_CODE := "sq-AL"
LCID_0484 := "Alsatian (France)"  ; gsw-FR
LCID_0484_CODE := "gsw-FR"
LCID_005E := "Amharic"  ; am
LCID_005E_CODE := "am"
LCID_045E := "Amharic (Ethiopia)"  ; am-ET
LCID_045E_CODE := "am-ET"
LCID_0001 := "Arabic"  ; ar
LCID_0001_CODE := "ar"
LCID_1401 := "Arabic (Algeria)"  ; ar-DZ
LCID_1401_CODE := "ar-DZ"
LCID_3C01 := "Arabic (Bahrain)"  ; ar-BH
LCID_3C01_CODE := "ar-BH"
LCID_0C01 := "Arabic (Egypt)"  ; ar-EG
LCID_0C01_CODE := "ar-EG"
LCID_0801 := "Arabic (Iraq)"  ; ar-IQ
LCID_0801_CODE := "ar-IQ"
LCID_2C01 := "Arabic (Jordan)"  ; ar-JO
LCID_2C01_CODE := "ar-JO"
LCID_3401 := "Arabic (Kuwait)"  ; ar-KW
LCID_3401_CODE := "ar-KW"
LCID_3001 := "Arabic (Lebanon)"  ; ar-LB
LCID_3001_CODE := "ar-LB"
LCID_1001 := "Arabic (Libya)"  ; ar-LY
LCID_1001_CODE := "ar-LY"
LCID_1801 := "Arabic (Morocco)"  ; ar-MA
LCID_1801_CODE := "ar-MA"
LCID_2001 := "Arabic (Oman)"  ; ar-OM
LCID_2001_CODE := "ar-OM"
LCID_4001 := "Arabic (Qatar)"  ; ar-QA
LCID_4001_CODE := "ar-QA"
LCID_0401 := "Arabic (Saudi Arabia)"  ; ar-SA
LCID_0401_CODE := "ar-SA"
LCID_2801 := "Arabic (Syria)"  ; ar-SY
LCID_2801_CODE := "ar-SY"
LCID_1C01 := "Arabic (Tunisia)"  ; ar-TN
LCID_1C01_CODE := "ar-TN"
LCID_3801 := "Arabic (United Arab Emirates)"  ; ar-AE
LCID_3801_CODE := "ar-AE"
LCID_2401 := "Arabic (Yemen)"  ; ar-YE
LCID_2401_CODE := "ar-YE"
LCID_002B := "Armenian"  ; hy
LCID_002B_CODE := "hy"
LCID_042B := "Armenian (Armenia)"  ; hy-AM
LCID_042B_CODE := "hy-AM"
LCID_004D := "Assamese"  ; as
LCID_004D_CODE := "as"
LCID_044D := "Assamese (India)"  ; as-IN
LCID_044D_CODE := "as-IN"
LCID_002C := "Azerbaijani"  ; az
LCID_002C_CODE := "az"
LCID_742C := "Azerbaijani (Cyrillic)"  ; az-Cyrl
LCID_742C_CODE := "az-Cyrl"
LCID_082C := "Azerbaijani (Cyrillic, Azerbaijan)"  ; az-Cyrl-AZ
LCID_082C_CODE := "az-Cyrl-AZ"
LCID_782C := "Azerbaijani (Latin)"  ; az-Latn
LCID_782C_CODE := "az-Latn"
LCID_042C := "Azerbaijani (Latin, Azerbaijan)"  ; az-Latn-AZ
LCID_042C_CODE := "az-Latn-AZ"
LCID_0045 := "Bangla"  ; bn
LCID_0045_CODE := "bn"
LCID_0845 := "Bangla (Bangladesh)"  ; bn-BD
LCID_0845_CODE := "bn-BD"
LCID_006D := "Bashkir"  ; ba
LCID_006D_CODE := "ba"
LCID_046D := "Bashkir (Russia)"  ; ba-RU
LCID_046D_CODE := "ba-RU"
LCID_002D := "Basque"  ; eu
LCID_002D_CODE := "eu"
LCID_042D := "Basque (Basque)"  ; eu-ES
LCID_042D_CODE := "eu-ES"
LCID_0023 := "Belarusian"  ; be
LCID_0023_CODE := "be"
LCID_0423 := "Belarusian (Belarus)"  ; be-BY
LCID_0423_CODE := "be-BY"
LCID_0445 := "Bengali (India)"  ; bn-IN
LCID_0445_CODE := "bn-IN"
LCID_781A := "Bosnian"  ; bs
LCID_781A_CODE := "bs"
LCID_641A := "Bosnian (Cyrillic)"  ; bs-Cyrl
LCID_641A_CODE := "bs-Cyrl"
LCID_201A := "Bosnian (Cyrillic, Bosnia and Herzegovina)"  ; bs-Cyrl-BA
LCID_201A_CODE := "bs-Cyrl-BA"
LCID_681A := "Bosnian (Latin)"  ; bs-Latn
LCID_681A_CODE := "bs-Latn"
LCID_141A := "Bosnian (Latin, Bosnia & Herzegovina)"  ; bs-Latn-BA
LCID_141A_CODE := "bs-Latn-BA"
LCID_007E := "Breton"  ; br
LCID_007E_CODE := "br"
LCID_047E := "Breton (France)"  ; br-FR
LCID_047E_CODE := "br-FR"
LCID_0002 := "Bulgarian"  ; bg
LCID_0002_CODE := "bg"
LCID_0402 := "Bulgarian (Bulgaria)"  ; bg-BG
LCID_0402_CODE := "bg-BG"
LCID_0055 := "Burmese"  ; my
LCID_0055_CODE := "my"
LCID_0455 := "Burmese (Myanmar)"  ; my-MM
LCID_0455_CODE := "my-MM"
LCID_0003 := "Catalan"  ; ca
LCID_0003_CODE := "ca"
LCID_0403 := "Catalan (Catalan)"  ; ca-ES
LCID_0403_CODE := "ca-ES"
LCID_005F := "Central Atlas Tamazight"  ; tzm
LCID_005F_CODE := "tzm"
LCID_045F := "Central Atlas Tamazight (Arabic, Morocco)"  ; tzm-Arab-MA
LCID_045F_CODE := "tzm-Arab-MA"
LCID_7C5F := "Central Atlas Tamazight (Latin)"  ; tzm-Latn
LCID_7C5F_CODE := "tzm-Latn"
LCID_085F := "Central Atlas Tamazight (Latin, Algeria)"  ; tzm-Latn-DZ
LCID_085F_CODE := "tzm-Latn-DZ"
LCID_785F := "Central Atlas Tamazight (Tifinagh)"  ; tzm-Tfng
LCID_785F_CODE := "tzm-Tfng"
LCID_105F := "Central Atlas Tamazight (Tifinagh, Morocco)"  ; tzm-Tfng-MA
LCID_105F_CODE := "tzm-Tfng-MA"
LCID_0092 := "Central Kurdish"  ; ku
LCID_0092_CODE := "ku"
LCID_7C92 := "Central Kurdish"  ; ku-Arab
LCID_7C92_CODE := "ku-Arab"
LCID_0492 := "Central Kurdish (Iraq)"  ; ku-Arab-IQ
LCID_0492_CODE := "ku-Arab-IQ"
LCID_005C := "Cherokee"  ; chr
LCID_005C_CODE := "chr"
LCID_7C5C := "Cherokee"  ; chr-Cher
LCID_7C5C_CODE := "chr-Cher"
LCID_045C := "Cherokee (Cherokee, United States)"  ; chr-Cher-US
LCID_045C_CODE := "chr-Cher-US"
LCID_7804 := "Chinese"  ; zh
LCID_7804_CODE := "zh"
LCID_0004 := "Chinese (Simplified)"  ; zh-Hans
LCID_0004_CODE := "zh-Hans"
LCID_0804 := "Chinese (Simplified, China)"  ; zh-CN
LCID_0804_CODE := "zh-CN"
LCID_1004 := "Chinese (Simplified, Singapore)"  ; zh-SG
LCID_1004_CODE := "zh-SG"
LCID_7C04 := "Chinese (Traditional)"  ; zh-Hant
LCID_7C04_CODE := "zh-Hant"
LCID_0C04 := "Chinese (Traditional, Hong Kong SAR)"  ; zh-HK
LCID_0C04_CODE := "zh-HK"
LCID_1404 := "Chinese (Traditional, Macao SAR)"  ; zh-MO
LCID_1404_CODE := "zh-MO"
LCID_0404 := "Chinese (Traditional, Taiwan)"  ; zh-TW
LCID_0404_CODE := "zh-TW"
LCID_0083 := "Corsican"  ; co
LCID_0083_CODE := "co"
LCID_0483 := "Corsican (France)"  ; co-FR
LCID_0483_CODE := "co-FR"
LCID_001A := "Croatian"  ; hr
LCID_001A_CODE := "hr"
LCID_101A := "Croatian (Bosnia & Herzegovina)"  ; hr-BA
LCID_101A_CODE := "hr-BA"
LCID_041A := "Croatian (Croatia)"  ; hr-HR
LCID_041A_CODE := "hr-HR"
LCID_0005 := "Czech"  ; cs
LCID_0005_CODE := "cs"
LCID_0405 := "Czech (Czechia)"  ; cs-CZ
LCID_0405_CODE := "cs-CZ"
LCID_0006 := "Danish"  ; da
LCID_0006_CODE := "da"
LCID_0406 := "Danish (Denmark)"  ; da-DK
LCID_0406_CODE := "da-DK"
LCID_0065 := "Divehi"  ; dv
LCID_0065_CODE := "dv"
LCID_0465 := "Divehi (Maldives)"  ; dv-MV
LCID_0465_CODE := "dv-MV"
LCID_0013 := "Dutch"  ; nl
LCID_0013_CODE := "nl"
LCID_0813 := "Dutch (Belgium)"  ; nl-BE
LCID_0813_CODE := "nl-BE"
LCID_0413 := "Dutch (Netherlands)"  ; nl-NL
LCID_0413_CODE := "nl-NL"
LCID_0C51 := "Dzongkha (Bhutan)"  ; dz-BT
LCID_0C51_CODE := "dz-BT"
LCID_0066 := "Edo"  ; bin
LCID_0066_CODE := "bin"
LCID_0466 := "Edo (Nigeria)"  ; bin-NG
LCID_0466_CODE := "bin-NG"
LCID_0009 := "English"  ; en
LCID_0009_CODE := "en"
LCID_0C09 := "English (Australia)"  ; en-AU
LCID_0C09_CODE := "en-AU"
LCID_2809 := "English (Belize)"  ; en-BZ
LCID_2809_CODE := "en-BZ"
LCID_1009 := "English (Canada)"  ; en-CA
LCID_1009_CODE := "en-CA"
LCID_2409 := "English (Caribbean)"  ; en-029
LCID_2409_CODE := "en-029"
LCID_3C09 := "English (Hong Kong SAR)"  ; en-HK
LCID_3C09_CODE := "en-HK"
LCID_4009 := "English (India)"  ; en-IN
LCID_4009_CODE := "en-IN"
LCID_3809 := "English (Indonesia)"  ; en-ID
LCID_3809_CODE := "en-ID"
LCID_1809 := "English (Ireland)"  ; en-IE
LCID_1809_CODE := "en-IE"
LCID_2009 := "English (Jamaica)"  ; en-JM
LCID_2009_CODE := "en-JM"
LCID_4409 := "English (Malaysia)"  ; en-MY
LCID_4409_CODE := "en-MY"
LCID_1409 := "English (New Zealand)"  ; en-NZ
LCID_1409_CODE := "en-NZ"
LCID_3409 := "English (Philippines)"  ; en-PH
LCID_3409_CODE := "en-PH"
LCID_4809 := "English (Singapore)"  ; en-SG
LCID_4809_CODE := "en-SG"
LCID_1C09 := "English (South Africa)"  ; en-ZA
LCID_1C09_CODE := "en-ZA"
LCID_2C09 := "English (Trinidad & Tobago)"  ; en-TT
LCID_2C09_CODE := "en-TT"
LCID_4C09 := "English (United Arab Emirates)"  ; en-AE
LCID_4C09_CODE := "en-AE"
LCID_0809 := "English (United Kingdom)"  ; en-GB
LCID_0809_CODE := "en-GB"
LCID_0409 := "English (United States)"  ; en-US
LCID_0409_CODE := "en-US"
LCID_3009 := "English (Zimbabwe)"  ; en-ZW
LCID_3009_CODE := "en-ZW"
LCID_0025 := "Estonian"  ; et
LCID_0025_CODE := "et"
LCID_0425 := "Estonian (Estonia)"  ; et-EE
LCID_0425_CODE := "et-EE"
LCID_0038 := "Faroese"  ; fo
LCID_0038_CODE := "fo"
LCID_0438 := "Faroese (Faroe Islands)"  ; fo-FO
LCID_0438_CODE := "fo-FO"
LCID_0064 := "Filipino"  ; fil
LCID_0064_CODE := "fil"
LCID_0464 := "Filipino (Philippines)"  ; fil-PH
LCID_0464_CODE := "fil-PH"
LCID_000B := "Finnish"  ; fi
LCID_000B_CODE := "fi"
LCID_040B := "Finnish (Finland)"  ; fi-FI
LCID_040B_CODE := "fi-FI"
LCID_000C := "French"  ; fr
LCID_000C_CODE := "fr"
LCID_080C := "French (Belgium)"  ; fr-BE
LCID_080C_CODE := "fr-BE"
LCID_2C0C := "French (Cameroon)"  ; fr-CM
LCID_2C0C_CODE := "fr-CM"
LCID_0C0C := "French (Canada)"  ; fr-CA
LCID_0C0C_CODE := "fr-CA"
LCID_1C0C := "French (Caribbean)"  ; fr-029
LCID_1C0C_CODE := "fr-029"
LCID_300C := "French (Côte d’Ivoire)"  ; fr-CI
LCID_300C_CODE := "fr-CI"
LCID_040C := "French (France)"  ; fr-FR
LCID_040C_CODE := "fr-FR"
LCID_3C0C := "French (Haiti)"  ; fr-HT
LCID_3C0C_CODE := "fr-HT"
LCID_140C := "French (Luxembourg)"  ; fr-LU
LCID_140C_CODE := "fr-LU"
LCID_340C := "French (Mali)"  ; fr-ML
LCID_340C_CODE := "fr-ML"
LCID_180C := "French (Monaco)"  ; fr-MC
LCID_180C_CODE := "fr-MC"
LCID_380C := "French (Morocco)"  ; fr-MA
LCID_380C_CODE := "fr-MA"
LCID_200C := "French (Réunion)"  ; fr-RE
LCID_200C_CODE := "fr-RE"
LCID_280C := "French (Senegal)"  ; fr-SN
LCID_280C_CODE := "fr-SN"
LCID_100C := "French (Switzerland)"  ; fr-CH
LCID_100C_CODE := "fr-CH"
LCID_240C := "French Congo (DRC)"  ; fr-CD
LCID_240C_CODE := "fr-CD"
LCID_0067 := "Fulah"  ; ff
LCID_0067_CODE := "ff"
LCID_7C67 := "Fulah (Latin)"  ; ff-Latn
LCID_7C67_CODE := "ff-Latn"
LCID_0467 := "Fulah (Latin, Nigeria)"  ; ff-Latn-NG
LCID_0467_CODE := "ff-Latn-NG"
LCID_0867 := "Fulah (Latin, Senegal)"  ; ff-Latn-SN
LCID_0867_CODE := "ff-Latn-SN"
LCID_0056 := "Galician"  ; gl
LCID_0056_CODE := "gl"
LCID_0456 := "Galician (Galician)"  ; gl-ES
LCID_0456_CODE := "gl-ES"
LCID_0037 := "Georgian"  ; ka
LCID_0037_CODE := "ka"
LCID_0437 := "Georgian (Georgia)"  ; ka-GE
LCID_0437_CODE := "ka-GE"
LCID_0007 := "German"  ; de
LCID_0007_CODE := "de"
LCID_0C07 := "German (Austria)"  ; de-AT
LCID_0C07_CODE := "de-AT"
LCID_0407 := "German (Germany)"  ; de-DE
LCID_0407_CODE := "de-DE"
LCID_1407 := "German (Liechtenstein)"  ; de-LI
LCID_1407_CODE := "de-LI"
LCID_1007 := "German (Luxembourg)"  ; de-LU
LCID_1007_CODE := "de-LU"
LCID_0807 := "German (Switzerland)"  ; de-CH
LCID_0807_CODE := "de-CH"
LCID_0008 := "Greek"  ; el
LCID_0008_CODE := "el"
LCID_0408 := "Greek (Greece)"  ; el-GR
LCID_0408_CODE := "el-GR"
LCID_0074 := "Guarani"  ; gn
LCID_0074_CODE := "gn"
LCID_0474 := "Guarani (Paraguay)"  ; gn-PY
LCID_0474_CODE := "gn-PY"
LCID_0047 := "Gujarati"  ; gu
LCID_0047_CODE := "gu"
LCID_0447 := "Gujarati (India)"  ; gu-IN
LCID_0447_CODE := "gu-IN"
LCID_0068 := "Hausa"  ; ha
LCID_0068_CODE := "ha"
LCID_7C68 := "Hausa (Latin)"  ; ha-Latn
LCID_7C68_CODE := "ha-Latn"
LCID_0468 := "Hausa (Latin, Nigeria)"  ; ha-Latn-NG
LCID_0468_CODE := "ha-Latn-NG"
LCID_0075 := "Hawaiian"  ; haw
LCID_0075_CODE := "haw"
LCID_0475 := "Hawaiian (United States)"  ; haw-US
LCID_0475_CODE := "haw-US"
LCID_000D := "Hebrew"  ; he
LCID_000D_CODE := "he"
LCID_040D := "Hebrew (Israel)"  ; he-IL
LCID_040D_CODE := "he-IL"
LCID_0039 := "Hindi"  ; hi
LCID_0039_CODE := "hi"
LCID_0439 := "Hindi (India)"  ; hi-IN
LCID_0439_CODE := "hi-IN"
LCID_000E := "Hungarian"  ; hu
LCID_000E_CODE := "hu"
LCID_040E := "Hungarian (Hungary)"  ; hu-HU
LCID_040E_CODE := "hu-HU"
LCID_0069 := "Ibibio"  ; ibb
LCID_0069_CODE := "ibb"
LCID_0469 := "Ibibio (Nigeria)"  ; ibb-NG
LCID_0469_CODE := "ibb-NG"
LCID_000F := "Icelandic"  ; is
LCID_000F_CODE := "is"
LCID_040F := "Icelandic (Iceland)"  ; is-IS
LCID_040F_CODE := "is-IS"
LCID_0070 := "Igbo"  ; ig
LCID_0070_CODE := "ig"
LCID_0470 := "Igbo (Nigeria)"  ; ig-NG
LCID_0470_CODE := "ig-NG"
LCID_0021 := "Indonesian"  ; id
LCID_0021_CODE := "id"
LCID_0421 := "Indonesian (Indonesia)"  ; id-ID
LCID_0421_CODE := "id-ID"
LCID_005D := "Inuktitut"  ; iu
LCID_005D_CODE := "iu"
LCID_7C5D := "Inuktitut (Latin)"  ; iu-Latn
LCID_7C5D_CODE := "iu-Latn"
LCID_085D := "Inuktitut (Latin, Canada)"  ; iu-Latn-CA
LCID_085D_CODE := "iu-Latn-CA"
LCID_785D := "Inuktitut (Syllabics)"  ; iu-Cans
LCID_785D_CODE := "iu-Cans"
LCID_045D := "Inuktitut (Syllabics, Canada)"  ; iu-Cans-CA
LCID_045D_CODE := "iu-Cans-CA"
LCID_003C := "Irish"  ; ga
LCID_003C_CODE := "ga"
LCID_083C := "Irish (Ireland)"  ; ga-IE
LCID_083C_CODE := "ga-IE"
LCID_0034 := "isiXhosa"  ; xh
LCID_0034_CODE := "xh"
LCID_0434 := "isiXhosa (South Africa)"  ; xh-ZA
LCID_0434_CODE := "xh-ZA"
LCID_0035 := "isiZulu"  ; zu
LCID_0035_CODE := "zu"
LCID_0435 := "isiZulu (South Africa)"  ; zu-ZA
LCID_0435_CODE := "zu-ZA"
LCID_0010 := "Italian"  ; it
LCID_0010_CODE := "it"
LCID_0410 := "Italian (Italy)"  ; it-IT
LCID_0410_CODE := "it-IT"
LCID_0810 := "Italian (Switzerland)"  ; it-CH
LCID_0810_CODE := "it-CH"
LCID_0011 := "Japanese"  ; ja
LCID_0011_CODE := "ja"
LCID_0411 := "Japanese (Japan)"  ; ja-JP
LCID_0411_CODE := "ja-JP"
LCID_006F := "Kalaallisut"  ; kl
LCID_006F_CODE := "kl"
LCID_046F := "Kalaallisut (Greenland)"  ; kl-GL
LCID_046F_CODE := "kl-GL"
LCID_004B := "Kannada"  ; kn
LCID_004B_CODE := "kn"
LCID_044B := "Kannada (India)"  ; kn-IN
LCID_044B_CODE := "kn-IN"
LCID_0071 := "Kanuri"  ; kr
LCID_0071_CODE := "kr"
LCID_0471 := "Kanuri (Latin, Nigeria)"  ; kr-Latn-NG
LCID_0471_CODE := "kr-Latn-NG"
LCID_0060 := "Kashmiri"  ; ks
LCID_0060_CODE := "ks"
LCID_0460 := "Kashmiri (Arabic)"  ; ks-Arab
LCID_0460_CODE := "ks-Arab"
LCID_1000 := "Kashmiri (Arabic)"  ; ks-Arab-IN
LCID_1000_CODE := "ks-Arab-IN"
LCID_0860 := "Kashmiri (Devanagari)"  ; ks-Deva-IN
LCID_0860_CODE := "ks-Deva-IN"
LCID_003F := "Kazakh"  ; kk
LCID_003F_CODE := "kk"
LCID_043F := "Kazakh (Kazakhstan)"  ; kk-KZ
LCID_043F_CODE := "kk-KZ"
LCID_0053 := "Khmer"  ; km
LCID_0053_CODE := "km"
LCID_0453 := "Khmer (Cambodia)"  ; km-KH
LCID_0453_CODE := "km-KH"
LCID_0087 := "Kinyarwanda"  ; rw
LCID_0087_CODE := "rw"
LCID_0487 := "Kinyarwanda (Rwanda)"  ; rw-RW
LCID_0487_CODE := "rw-RW"
LCID_0041 := "Kiswahili"  ; sw
LCID_0041_CODE := "sw"
LCID_0441 := "Kiswahili (Kenya)"  ; sw-KE
LCID_0441_CODE := "sw-KE"
LCID_0057 := "Konkani"  ; kok
LCID_0057_CODE := "kok"
LCID_0457 := "Konkani (India)"  ; kok-IN
LCID_0457_CODE := "kok-IN"
LCID_0012 := "Korean"  ; ko
LCID_0012_CODE := "ko"
LCID_0412 := "Korean (Korea)"  ; ko-KR
LCID_0412_CODE := "ko-KR"
LCID_0040 := "Kyrgyz"  ; ky
LCID_0040_CODE := "ky"
LCID_0440 := "Kyrgyz (Kyrgyzstan)"  ; ky-KG
LCID_0440_CODE := "ky-KG"
LCID_0086 := "Kʼicheʼ"  ; quc
LCID_0086_CODE := "quc"
LCID_7C86 := "Kʼicheʼ (Latin)"  ; quc-Latn
LCID_7C86_CODE := "quc-Latn"
LCID_0486 := "Kʼicheʼ (Latin, Guatemala)"  ; quc-Latn-GT
LCID_0486_CODE := "quc-Latn-GT"
LCID_0054 := "Lao"  ; lo
LCID_0054_CODE := "lo"
LCID_0454 := "Lao (Laos)"  ; lo-LA
LCID_0454_CODE := "lo-LA"
LCID_0076 := "Latin"  ; la
LCID_0076_CODE := "la"
LCID_0476 := "Latin (Vatican City)"  ; la-VA
LCID_0476_CODE := "la-VA"
LCID_0026 := "Latvian"  ; lv
LCID_0026_CODE := "lv"
LCID_0426 := "Latvian (Latvia)"  ; lv-LV
LCID_0426_CODE := "lv-LV"
LCID_0027 := "Lithuanian"  ; lt
LCID_0027_CODE := "lt"
LCID_0427 := "Lithuanian (Lithuania)"  ; lt-LT
LCID_0427_CODE := "lt-LT"
LCID_7C2E := "Lower Sorbian"  ; dsb
LCID_7C2E_CODE := "dsb"
LCID_082E := "Lower Sorbian (Germany)"  ; dsb-DE
LCID_082E_CODE := "dsb-DE"
LCID_006E := "Luxembourgish"  ; lb
LCID_006E_CODE := "lb"
LCID_046E := "Luxembourgish (Luxembourg)"  ; lb-LU
LCID_046E_CODE := "lb-LU"
LCID_002F := "Macedonian"  ; mk
LCID_002F_CODE := "mk"
LCID_042F := "Macedonian (North Macedonia)"  ; mk-MK
LCID_042F_CODE := "mk-MK"
LCID_003E := "Malay"  ; ms
LCID_003E_CODE := "ms"
LCID_083E := "Malay (Brunei)"  ; ms-BN
LCID_083E_CODE := "ms-BN"
LCID_043E := "Malay (Malaysia)"  ; ms-MY
LCID_043E_CODE := "ms-MY"
LCID_004C := "Malayalam"  ; ml
LCID_004C_CODE := "ml"
LCID_044C := "Malayalam (India)"  ; ml-IN
LCID_044C_CODE := "ml-IN"
LCID_003A := "Maltese"  ; mt
LCID_003A_CODE := "mt"
LCID_043A := "Maltese (Malta)"  ; mt-MT
LCID_043A_CODE := "mt-MT"
LCID_0058 := "Manipuri"  ; mni
LCID_0058_CODE := "mni"
LCID_0458 := "Manipuri (Bangla, India)"  ; mni-IN
LCID_0458_CODE := "mni-IN"
LCID_0081 := "Maori"  ; mi
LCID_0081_CODE := "mi"
LCID_0481 := "Maori (New Zealand)"  ; mi-NZ
LCID_0481_CODE := "mi-NZ"
LCID_007A := "Mapuche"  ; arn
LCID_007A_CODE := "arn"
LCID_047A := "Mapuche (Chile)"  ; arn-CL
LCID_047A_CODE := "arn-CL"
LCID_004E := "Marathi"  ; mr
LCID_004E_CODE := "mr"
LCID_044E := "Marathi (India)"  ; mr-IN
LCID_044E_CODE := "mr-IN"
LCID_007C := "Mohawk"  ; moh
LCID_007C_CODE := "moh"
LCID_047C := "Mohawk (Canada)"  ; moh-CA
LCID_047C_CODE := "moh-CA"
LCID_0050 := "Mongolian"  ; mn
LCID_0050_CODE := "mn"
LCID_7850 := "Mongolian"  ; mn-Cyrl
LCID_7850_CODE := "mn-Cyrl"
LCID_0450 := "Mongolian (Mongolia)"  ; mn-MN
LCID_0450_CODE := "mn-MN"
LCID_7C50 := "Mongolian (Traditional Mongolian)"  ; mn-Mong
LCID_7C50_CODE := "mn-Mong"
LCID_0850 := "Mongolian (Traditional Mongolian, China)"  ; mn-Mong-CN
LCID_0850_CODE := "mn-Mong-CN"
LCID_0C50 := "Mongolian (Traditional Mongolian, Mongolia)"  ; mn-Mong-MN
LCID_0C50_CODE := "mn-Mong-MN"
LCID_0061 := "Nepali"  ; ne
LCID_0061_CODE := "ne"
LCID_0861 := "Nepali (India)"  ; ne-IN
LCID_0861_CODE := "ne-IN"
LCID_0461 := "Nepali (Nepal)"  ; ne-NP
LCID_0461_CODE := "ne-NP"
LCID_003B := "Northern Sami"  ; se
LCID_003B_CODE := "se"
LCID_0014 := "Norwegian"  ; no
LCID_0014_CODE := "no"
LCID_7C14 := "Norwegian Bokmål"  ; nb
LCID_7C14_CODE := "nb"
LCID_0414 := "Norwegian Bokmål (Norway)"  ; nb-NO
LCID_0414_CODE := "nb-NO"
LCID_7814 := "Norwegian Nynorsk"  ; nn
LCID_7814_CODE := "nn"
LCID_0814 := "Norwegian Nynorsk (Norway)"  ; nn-NO
LCID_0814_CODE := "nn-NO"
LCID_0082 := "Occitan"  ; oc
LCID_0082_CODE := "oc"
LCID_0482 := "Occitan (France)"  ; oc-FR
LCID_0482_CODE := "oc-FR"
LCID_0048 := "Odia"  ; or
LCID_0048_CODE := "or"
LCID_0448 := "Odia (India)"  ; or-IN
LCID_0448_CODE := "or-IN"
LCID_0072 := "Oromo"  ; om
LCID_0072_CODE := "om"
LCID_0472 := "Oromo (Ethiopia)"  ; om-ET
LCID_0472_CODE := "om-ET"
LCID_0079 := "Papiamento"  ; pap
LCID_0079_CODE := "pap"
LCID_0479 := "Papiamento (Caribbean)"  ; pap-029
LCID_0479_CODE := "pap-029"
LCID_0063 := "Pashto"  ; ps
LCID_0063_CODE := "ps"
LCID_0463 := "Pashto (Afghanistan)"  ; ps-AF
LCID_0463_CODE := "ps-AF"
LCID_0029 := "Persian"  ; fa
LCID_0029_CODE := "fa"
LCID_008C := "Persian"  ; fa
LCID_008C_CODE := "fa"
LCID_048C := "Persian (Afghanistan)"  ; fa-AF
LCID_048C_CODE := "fa-AF"
LCID_0429 := "Persian (Iran)"  ; fa-IR
LCID_0429_CODE := "fa-IR"
LCID_0015 := "Polish"  ; pl
LCID_0015_CODE := "pl"
LCID_0415 := "Polish (Poland)"  ; pl-PL
LCID_0415_CODE := "pl-PL"
LCID_0016 := "Portuguese"  ; pt
LCID_0016_CODE := "pt"
LCID_0416 := "Portuguese (Brazil)"  ; pt-BR
LCID_0416_CODE := "pt-BR"
LCID_0816 := "Portuguese (Portugal)"  ; pt-PT
LCID_0816_CODE := "pt-PT"
LCID_05FE := "Pseudo (Pseudo Asia)"  ; qps-ploca
LCID_05FE_CODE := "qps-ploca"
LCID_09FF := "Pseudo (Pseudo Mirrored)"  ; qps-plocm
LCID_09FF_CODE := "qps-plocm"
LCID_0901 := "Pseudo (Pseudo Selfhost)"  ; qps-Latn-x-sh
LCID_0901_CODE := "qps-Latn-x-sh"
LCID_0501 := "Pseudo (Pseudo)"  ; qps-ploc
LCID_0501_CODE := "qps-ploc"
LCID_0046 := "Punjabi"  ; pa
LCID_0046_CODE := "pa"
LCID_7C46 := "Punjabi"  ; pa-Arab
LCID_7C46_CODE := "pa-Arab"
LCID_0446 := "Punjabi (India)"  ; pa-IN
LCID_0446_CODE := "pa-IN"
LCID_0846 := "Punjabi (Pakistan)"  ; pa-Arab-PK
LCID_0846_CODE := "pa-Arab-PK"
LCID_006B := "Quechua"  ; quz
LCID_006B_CODE := "quz"
LCID_046B := "Quechua (Bolivia)"  ; quz-BO
LCID_046B_CODE := "quz-BO"
LCID_086B := "Quechua (Ecuador)"  ; quz-EC
LCID_086B_CODE := "quz-EC"
LCID_0C6B := "Quechua (Peru)"  ; quz-PE
LCID_0C6B_CODE := "quz-PE"
LCID_0018 := "Romanian"  ; ro
LCID_0018_CODE := "ro"
LCID_0818 := "Romanian (Moldova)"  ; ro-MD
LCID_0818_CODE := "ro-MD"
LCID_0418 := "Romanian (Romania)"  ; ro-RO
LCID_0418_CODE := "ro-RO"
LCID_0017 := "Romansh"  ; rm
LCID_0017_CODE := "rm"
LCID_0417 := "Romansh (Switzerland)"  ; rm-CH
LCID_0417_CODE := "rm-CH"
LCID_0019 := "Russian"  ; ru
LCID_0019_CODE := "ru"
LCID_0819 := "Russian (Moldova)"  ; ru-MD
LCID_0819_CODE := "ru-MD"
LCID_0419 := "Russian (Russia)"  ; ru-RU
LCID_0419_CODE := "ru-RU"
LCID_0085 := "Sakha"  ; sah
LCID_0085_CODE := "sah"
LCID_0485 := "Sakha (Russia)"  ; sah-RU
LCID_0485_CODE := "sah-RU"
LCID_703B := "Sami (Inari)"  ; smn
LCID_703B_CODE := "smn"
LCID_7C3B := "Sami (Lule)"  ; smj
LCID_7C3B_CODE := "smj"
LCID_743B := "Sami (Skolt)"  ; sms
LCID_743B_CODE := "sms"
LCID_783B := "Sami (Southern)"  ; sma
LCID_783B_CODE := "sma"
LCID_243B := "Sami, Inari (Finland)"  ; smn-FI
LCID_243B_CODE := "smn-FI"
LCID_103B := "Sami, Lule (Norway)"  ; smj-NO
LCID_103B_CODE := "smj-NO"
LCID_143B := "Sami, Lule (Sweden)"  ; smj-SE
LCID_143B_CODE := "smj-SE"
LCID_0C3B := "Sami, Northern (Finland)"  ; se-FI
LCID_0C3B_CODE := "se-FI"
LCID_043B := "Sami, Northern (Norway)"  ; se-NO
LCID_043B_CODE := "se-NO"
LCID_083B := "Sami, Northern (Sweden)"  ; se-SE
LCID_083B_CODE := "se-SE"
LCID_203B := "Sami, Skolt (Finland)"  ; sms-FI
LCID_203B_CODE := "sms-FI"
LCID_183B := "Sami, Southern (Norway)"  ; sma-NO
LCID_183B_CODE := "sma-NO"
LCID_1C3B := "Sami, Southern (Sweden)"  ; sma-SE
LCID_1C3B_CODE := "sma-SE"
LCID_004F := "Sanskrit"  ; sa
LCID_004F_CODE := "sa"
LCID_044F := "Sanskrit (India)"  ; sa-IN
LCID_044F_CODE := "sa-IN"
LCID_0091 := "Scottish Gaelic"  ; gd
LCID_0091_CODE := "gd"
LCID_0491 := "Scottish Gaelic (United Kingdom)"  ; gd-GB
LCID_0491_CODE := "gd-GB"
LCID_7C1A := "Serbian"  ; sr
LCID_7C1A_CODE := "sr"
LCID_6C1A := "Serbian (Cyrillic)"  ; sr-Cyrl
LCID_6C1A_CODE := "sr-Cyrl"
LCID_1C1A := "Serbian (Cyrillic, Bosnia and Herzegovina)"  ; sr-Cyrl-BA
LCID_1C1A_CODE := "sr-Cyrl-BA"
LCID_301A := "Serbian (Cyrillic, Montenegro)"  ; sr-Cyrl-ME
LCID_301A_CODE := "sr-Cyrl-ME"
LCID_0C1A := "Serbian (Cyrillic, Serbia and Montenegro (Former))"  ; sr-Cyrl-CS
LCID_0C1A_CODE := "sr-Cyrl-CS"
LCID_281A := "Serbian (Cyrillic, Serbia)"  ; sr-Cyrl-RS
LCID_281A_CODE := "sr-Cyrl-RS"
LCID_701A := "Serbian (Latin)"  ; sr-Latn
LCID_701A_CODE := "sr-Latn"
LCID_181A := "Serbian (Latin, Bosnia & Herzegovina)"  ; sr-Latn-BA
LCID_181A_CODE := "sr-Latn-BA"
LCID_2C1A := "Serbian (Latin, Montenegro)"  ; sr-Latn-ME
LCID_2C1A_CODE := "sr-Latn-ME"
LCID_081A := "Serbian (Latin, Serbia and Montenegro (Former))"  ; sr-Latn-CS
LCID_081A_CODE := "sr-Latn-CS"
LCID_241A := "Serbian (Latin, Serbia)"  ; sr-Latn-RS
LCID_241A_CODE := "sr-Latn-RS"
LCID_0030 := "Sesotho"  ; st
LCID_0030_CODE := "st"
LCID_0430 := "Sesotho (South Africa)"  ; st-ZA
LCID_0430_CODE := "st-ZA"
LCID_006C := "Sesotho sa Leboa"  ; nso
LCID_006C_CODE := "nso"
LCID_046C := "Sesotho sa Leboa (South Africa)"  ; nso-ZA
LCID_046C_CODE := "nso-ZA"
LCID_0032 := "Setswana"  ; tn
LCID_0032_CODE := "tn"
LCID_0832 := "Setswana (Botswana)"  ; tn-BW
LCID_0832_CODE := "tn-BW"
LCID_0432 := "Setswana (South Africa)"  ; tn-ZA
LCID_0432_CODE := "tn-ZA"
LCID_0059 := "Sindhi"  ; sd
LCID_0059_CODE := "sd"
LCID_7C59 := "Sindhi"  ; sd-Arab
LCID_7C59_CODE := "sd-Arab"
LCID_0459 := "Sindhi (Devanagari, India)"  ; sd-Deva-IN
LCID_0459_CODE := "sd-Deva-IN"
LCID_0859 := "Sindhi (Pakistan)"  ; sd-Arab-PK
LCID_0859_CODE := "sd-Arab-PK"
LCID_005B := "Sinhala"  ; si
LCID_005B_CODE := "si"
LCID_045B := "Sinhala (Sri Lanka)"  ; si-LK
LCID_045B_CODE := "si-LK"
LCID_001B := "Slovak"  ; sk
LCID_001B_CODE := "sk"
LCID_041B := "Slovak (Slovakia)"  ; sk-SK
LCID_041B_CODE := "sk-SK"
LCID_0024 := "Slovenian"  ; sl
LCID_0024_CODE := "sl"
LCID_0424 := "Slovenian (Slovenia)"  ; sl-SI
LCID_0424_CODE := "sl-SI"
LCID_0077 := "Somali"  ; so
LCID_0077_CODE := "so"
LCID_0477 := "Somali (Somalia)"  ; so-SO
LCID_0477_CODE := "so-SO"
LCID_000A := "Spanish"  ; es
LCID_000A_CODE := "es"
LCID_2C0A := "Spanish (Argentina)"  ; es-AR
LCID_2C0A_CODE := "es-AR"
LCID_400A := "Spanish (Bolivia)"  ; es-BO
LCID_400A_CODE := "es-BO"
LCID_340A := "Spanish (Chile)"  ; es-CL
LCID_340A_CODE := "es-CL"
LCID_240A := "Spanish (Colombia)"  ; es-CO
LCID_240A_CODE := "es-CO"
LCID_140A := "Spanish (Costa Rica)"  ; es-CR
LCID_140A_CODE := "es-CR"
LCID_5C0A := "Spanish (Cuba)"  ; es-CU
LCID_5C0A_CODE := "es-CU"
LCID_1C0A := "Spanish (Dominican Republic)"  ; es-DO
LCID_1C0A_CODE := "es-DO"
LCID_300A := "Spanish (Ecuador)"  ; es-EC
LCID_300A_CODE := "es-EC"
LCID_440A := "Spanish (El Salvador)"  ; es-SV
LCID_440A_CODE := "es-SV"
LCID_100A := "Spanish (Guatemala)"  ; es-GT
LCID_100A_CODE := "es-GT"
LCID_480A := "Spanish (Honduras)"  ; es-HN
LCID_480A_CODE := "es-HN"
LCID_580A := "Spanish (Latin America)"  ; es-419
LCID_580A_CODE := "es-419"
LCID_080A := "Spanish (Mexico)"  ; es-MX
LCID_080A_CODE := "es-MX"
LCID_4C0A := "Spanish (Nicaragua)"  ; es-NI
LCID_4C0A_CODE := "es-NI"
LCID_180A := "Spanish (Panama)"  ; es-PA
LCID_180A_CODE := "es-PA"
LCID_3C0A := "Spanish (Paraguay)"  ; es-PY
LCID_3C0A_CODE := "es-PY"
LCID_280A := "Spanish (Peru)"  ; es-PE
LCID_280A_CODE := "es-PE"
LCID_500A := "Spanish (Puerto Rico)"  ; es-PR
LCID_500A_CODE := "es-PR"
LCID_0C0A := "Spanish (Spain, International Sort)"  ; es-ES
LCID_0C0A_CODE := "es-ES"
LCID_040A := "Spanish (Spain, Traditional Sort)"  ; es-ES_tradnl
LCID_040A_CODE := "es-ES_tradnl"
LCID_540A := "Spanish (United States)"  ; es-US
LCID_540A_CODE := "es-US"
LCID_380A := "Spanish (Uruguay)"  ; es-UY
LCID_380A_CODE := "es-UY"
LCID_200A := "Spanish (Venezuela)"  ; es-VE
LCID_200A_CODE := "es-VE"
LCID_001D := "Swedish"  ; sv
LCID_001D_CODE := "sv"
LCID_081D := "Swedish (Finland)"  ; sv-FI
LCID_081D_CODE := "sv-FI"
LCID_041D := "Swedish (Sweden)"  ; sv-SE
LCID_041D_CODE := "sv-SE"
LCID_0084 := "Swiss German"  ; gsw
LCID_0084_CODE := "gsw"
LCID_005A := "Syriac"  ; syr
LCID_005A_CODE := "syr"
LCID_045A := "Syriac (Syria)"  ; syr-SY
LCID_045A_CODE := "syr-SY"
LCID_0028 := "Tajik"  ; tg
LCID_0028_CODE := "tg"
LCID_7C28 := "Tajik (Cyrillic)"  ; tg-Cyrl
LCID_7C28_CODE := "tg-Cyrl"
LCID_0428 := "Tajik (Cyrillic, Tajikistan)"  ; tg-Cyrl-TJ
LCID_0428_CODE := "tg-Cyrl-TJ"
LCID_0049 := "Tamil"  ; ta
LCID_0049_CODE := "ta"
LCID_0449 := "Tamil (India)"  ; ta-IN
LCID_0449_CODE := "ta-IN"
LCID_0849 := "Tamil (Sri Lanka)"  ; ta-LK
LCID_0849_CODE := "ta-LK"
LCID_0044 := "Tatar"  ; tt
LCID_0044_CODE := "tt"
LCID_0444 := "Tatar (Russia)"  ; tt-RU
LCID_0444_CODE := "tt-RU"
LCID_004A := "Telugu"  ; te
LCID_004A_CODE := "te"
LCID_044A := "Telugu (India)"  ; te-IN
LCID_044A_CODE := "te-IN"
LCID_001E := "Thai"  ; th
LCID_001E_CODE := "th"
LCID_041E := "Thai (Thailand)"  ; th-TH
LCID_041E_CODE := "th-TH"
LCID_0051 := "Tibetan"  ; bo
LCID_0051_CODE := "bo"
LCID_0451 := "Tibetan (China)"  ; bo-CN
LCID_0451_CODE := "bo-CN"
LCID_0073 := "Tigrinya"  ; ti
LCID_0073_CODE := "ti"
LCID_0873 := "Tigrinya (Eritrea)"  ; ti-ER
LCID_0873_CODE := "ti-ER"
LCID_0473 := "Tigrinya (Ethiopia)"  ; ti-ET
LCID_0473_CODE := "ti-ET"
LCID_001F := "Turkish"  ; tr
LCID_001F_CODE := "tr"
LCID_041F := "Turkish (Turkey)"  ; tr-TR
LCID_041F_CODE := "tr-TR"
LCID_0042 := "Turkmen"  ; tk
LCID_0042_CODE := "tk"
LCID_0442 := "Turkmen (Turkmenistan)"  ; tk-TM
LCID_0442_CODE := "tk-TM"
LCID_0022 := "Ukrainian"  ; uk
LCID_0022_CODE := "uk"
LCID_0422 := "Ukrainian (Ukraine)"  ; uk-UA
LCID_0422_CODE := "uk-UA"
LCID_002E := "Upper Sorbian"  ; hsb
LCID_002E_CODE := "hsb"
LCID_042E := "Upper Sorbian (Germany)"  ; hsb-DE
LCID_042E_CODE := "hsb-DE"
LCID_0020 := "Urdu"  ; ur
LCID_0020_CODE := "ur"
LCID_0820 := "Urdu (India)"  ; ur-IN
LCID_0820_CODE := "ur-IN"
LCID_0420 := "Urdu (Pakistan)"  ; ur-PK
LCID_0420_CODE := "ur-PK"
LCID_0080 := "Uyghur"  ; ug
LCID_0080_CODE := "ug"
LCID_0480 := "Uyghur (China)"  ; ug-CN
LCID_0480_CODE := "ug-CN"
LCID_0043 := "Uzbek"  ; uz
LCID_0043_CODE := "uz"
LCID_7843 := "Uzbek (Cyrillic)"  ; uz-Cyrl
LCID_7843_CODE := "uz-Cyrl"
LCID_0843 := "Uzbek (Cyrillic, Uzbekistan)"  ; uz-Cyrl-UZ
LCID_0843_CODE := "uz-Cyrl-UZ"
LCID_7C43 := "Uzbek (Latin)"  ; uz-Latn
LCID_7C43_CODE := "uz-Latn"
LCID_0443 := "Uzbek (Latin, Uzbekistan)"  ; uz-Latn-UZ
LCID_0443_CODE := "uz-Latn-UZ"
LCID_0803 := "Valencian (Spain)"  ; ca-ES-valencia
LCID_0803_CODE := "ca-ES-valencia"
LCID_0033 := "Venda"  ; ve
LCID_0033_CODE := "ve"
LCID_0433 := "Venda (South Africa)"  ; ve-ZA
LCID_0433_CODE := "ve-ZA"
LCID_002A := "Vietnamese"  ; vi
LCID_002A_CODE := "vi"
LCID_042A := "Vietnamese (Vietnam)"  ; vi-VN
LCID_042A_CODE := "vi-VN"
LCID_0052 := "Welsh"  ; cy
LCID_0052_CODE := "cy"
LCID_0452 := "Welsh (United Kingdom)"  ; cy-GB
LCID_0452_CODE := "cy-GB"
LCID_0062 := "Western Frisian"  ; fy
LCID_0062_CODE := "fy"
LCID_0462 := "Western Frisian (Netherlands)"  ; fy-NL
LCID_0462_CODE := "fy-NL"
LCID_0088 := "Wolof"  ; wo
LCID_0088_CODE := "wo"
LCID_0488 := "Wolof (Senegal)"  ; wo-SN
LCID_0488_CODE := "wo-SN"
LCID_0031 := "Xitsonga"  ; ts
LCID_0031_CODE := "ts"
LCID_0431 := "Xitsonga (South Africa)"  ; ts-ZA
LCID_0431_CODE := "ts-ZA"
LCID_0078 := "Yi"  ; ii
LCID_0078_CODE := "ii"
LCID_0478 := "Yi (China)"  ; ii-CN
LCID_0478_CODE := "ii-CN"
LCID_003D := "Yiddish"  ; yi
LCID_003D_CODE := "yi"
LCID_043D := "Yiddish (World)"  ; yi-001
LCID_043D_CODE := "yi-001"
LCID_006A := "Yoruba"  ; yo
LCID_006A_CODE := "yo"
LCID_046A := "Yoruba (Nigeria)"  ; yo-NG
LCID_046A_CODE := "yo-NG"

global CLX_I18N_DEFAULT_LANG := LCID_%A_Language%_CODE  ; Get the name of the system's default language.
; MsgBox %CLX_I18N_DEFAULT_LANG%  ; Display the language name.

GetKeyboardLanguage()
{
    if !KBLayout := DllCall("user32.dll\GetKeyboardLayout")
        return false
    return KBLayout & 0xFFFF
}

t(s)
{
    global CLX_Lang

    key := s
    defaultValue := s
    explain := s

    ; for dev, autotranslate
    ; run node "prompts/translate-en.md"
    if (lang == ""){
        lang := CLX_Lang
    }
    if (!lang) {
        lang := "auto"
    }
    if ( lang == "auto" ) {
        ; alang := GetKeyboardLanguage()
        lang := CLX_I18N_DEFAULT_LANG
    }
    return i18n_translated(lang, key)
}

i18n_translated(lang, key)
{
    ; user translation
    ; translated := CLX_ConfigGet("lang-" . lang, key, "")
    ; if (translated) {
    ;     return translated
    ; }
    ; system translation
    translated := CLX_i18n_ConfigGet("lang-" . lang, key, "")
    if (translated) {
        return translated
    }

    question := ""
    question .= ">>> TASK: translate to " . lang . ", no explains, output only transcript" . "`n"
    question .= ">>> INPUT:"
    question .= key . "`n"
    question .= ">>> OUTPUT:"

    global brainstorm_origin
    if (!brainstorm_origin) {
        brainstorm_origin := CLX_Config("BrainStorm", "Website", "https://brainstorm.snomiao.com")
    }
    endpoint := brainstorm_origin . "/ai/translator?ret=text"
    xhr := ComObjCreate("Msxml2.XMLHTTP")
    xhr.Open("POST", endpoint)
    xhr.setRequestHeader("Authorization", "Bearer " . brainstormApiKey)
    xhr.onreadystatechange := Func("i18n_brainstorm_translatePostResult").Bind(lang, key, xhr)
    xhr.Send(question)

    return key . "…"
    ; return "…[" . key . "]"
}
i18n_brainstorm_translatePostResult(lang, key, xhr)
{
    if (xhr.readyState != 4)
        return
    if (xhr.status != 200) {
        if (xhr.status == 429) {
            MsgBox, % xhr.responseText " Please wait a moment then try again."
        } else if (xhr.status == 500) {
            ; ignore 500 error
            return
        }
        ; ignore translation error
        ; MsgBox, % xhr.status . " " . xhr.responseText . " " . ("未知错误 / Unknown Error")
        return
    }
    global transcript := xhr.responseText
    if (!transcript) {
        ; ignore translation error
        ; MsgBox, fail to ask ai
        return
    }
    TrayTip, % "CapsLockX i18n [" . lang . "]", % key "=>" transcript,
    CLX_i18n_ConfigSet("lang-" . lang, key, transcript)
    ; CLX_ConfigSet("lang-" . lang, key, transcript)
}

i18n_changeLanguage(lang := "auto")
{
    CLX_Lang := lang
    if (!lang) {
        lang:="auto"
    }
    CLX_ConfigSet("Core", "Language", lang)
}
CLX_i18n_ConfigGet(field, varName, defaultValue)
{
    encodedKey := CLX_i18n_ConfigEnocde(varName)
    global CLX_ConfigChangedTickCount
    CLX_ConfigChangedTickCount := A_TickCount
    ; user locales
    ; global CLX_ConfigDir
    ; CONVERT_FILE_TO_UTF16_WITH_BOM_ENCODING(CLX_ConfigDir . "/" . field . ".ini")
    ; IniRead, content, % CLX_ConfigDir . "/" . field . ".ini", %field%, % encodedKey, %defaultValue%
    ; if (content == "ERROR") {
    ;     content := ""
    ; }
    ; if (content) {
    ;     return CLX_i18n_ConfigDecode(content)
    ; }
    ; clx pre-installed locales v2
    global CLX_ConfigDir
    CONVERT_FILE_TO_UTF16_WITH_BOM_ENCODING(CLX_ConfigDir . "/" . field . ".ini")
    IniRead, content, % CLX_i18nConfigDir . "/" . field . ".ini", %field%, % encodedKey, %defaultValue%
    if (content == "ERROR") {
        content := ""
    }
    if (content) {
        return CLX_i18n_ConfigDecode(content)
    }
    ; clx pre-installed locales
    CONVERT_FILE_TO_UTF16_WITH_BOM_ENCODING(CLX_i18nConfigPath)
    IniRead, content, % CLX_i18nConfigPath, %field%, % encodedKey, %defaultValue%
    if (content == "ERROR") {
        content := ""
    }
    if (content) {
        return CLX_i18n_ConfigDecode(content)
    }
}

CLX_i18n_ConfigSet(field, varName, value)
{
    encodedKey := CLX_i18n_ConfigEnocde(varName)
    encodedValue := CLX_i18n_ConfigEnocde(value)
    global CLX_ConfigChangedTickCount
    CLX_ConfigChangedTickCount := A_TickCount
    global CLX_ConfigDir
    ; save to lang.ini if installed by source code (where has .git folder)
    if (CLX_i18n_IsCLX_InstalledByGitClone){
        ; v2
        IniSave(encodedValue, CLX_i18nConfigDir . "/" . field . ".ini", field, encodedKey)
        ; v1
        ; IniSave(encodedValue, CLX_i18nConfigPath, field, encodedKey)
    }else{
        IniSave(encodedValue, CLX_ConfigDir . "/" . field . ".ini", field, encodedKey)
    }

    ; CONVERT_FILE_TO_UTF16_WITH_BOM_ENCODING(CLX_ConfigDir)
}
CLX_i18n_ConfigEnocde(str){
    str := RegExReplace(str, "\\", "\\")
    str := RegExReplace(str, "`r", "\r")
    str := RegExReplace(str, "`n", "\n")
    str := RegExReplace(str, "=", "\e")
    return str
}
CLX_i18n_ConfigDecode(str){
    str := RegExReplace(str, "\\e", "=")
    str := RegExReplace(str, "\\n", "`n")
    str := RegExReplace(str, "\\r", "`r")
    str := RegExReplace(str, "\\\\", "\")
    return str
}