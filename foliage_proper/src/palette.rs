//! Palette families beyond the Tailwind set, on the same eleven-step scale.
//!
//! [`bevy_color::palettes::tailwind`] carries seventeen chromatic hues and five neutrals. This
//! adds the ones a UI reaches for and finds missing: warm earths (`clay`, `terracotta`, `sepia`,
//! `taupe`), metals (`gold`, `bronze`, `copper`), the yellow-green arc Tailwind jumps over
//! between `yellow` and `lime` (`olive`, `chartreuse`), muted greens (`sage`, `moss`, `forest`),
//! quiet tinted greys (`mist`, `steel`), and the purple-to-red stretch (`lilac`, `plum`, `mauve`,
//! `wine`, `crimson`). Reached like any other family -- [`Color::olive`](crate::Color).
//!
//! # Where the numbers come from
//!
//! Nothing here is picked by eye. Two independent inputs decide every step:
//!
//! **The ramp's shape comes from Tailwind.** Lightness is read back out of Tailwind's own
//! families, which turn out to run two different curves -- `stone(700)` is L 0.372 while the
//! average chromatic 700 is L 0.515 -- so a family interpolates between them by how chromatic it
//! is. A near-grey like `mist` lands on the neutral curve, where it will be compared against
//! `zinc`; a saturated one like `crimson` lands on the chromatic curve beside `red`. Chroma
//! follows Tailwind's own arc over the ramp, peaking at 500. Hue is held across all eleven steps
//! so a family does not drift toward another at its dark end, and any step falling outside sRGB
//! has its chroma backed off until it fits.
//!
//! **Each family's identity comes from its name.** The hue angle and the chroma multiplier are
//! not chosen -- they are read off a reference hex for that name (the CSS named colour where one
//! exists, the conventional value where it does not), by converting it to OKLCH and matching its
//! chroma at whichever step sits at its lightness. Every family's comment records that reference,
//! the step it anchors at, and how far the generated step lands from it in OKLab distance. The
//! median is 0.023 and the worst is `chartreuse` at 0.066, which is unreachable because #7fff00
//! sits on the sRGB gamut boundary and a hue-constant ramp cannot get to a corner.
//!
//! To retune a family, change its reference hex and regenerate; do not hand-edit a step, or the
//! ramp stops meaning what the comment above it says.

use bevy_color::Srgba;

// sand -- #c2b280 at 400, hue 92.4, chroma x0.42 (off by 0.031)
pub const SAND_50: Srgba = Srgba::rgb(0.98269845, 0.97604834, 0.95468361);
pub const SAND_100: Srgba = Srgba::rgb(0.96467274, 0.94924234, 0.89999811);
pub const SAND_200: Srgba = Srgba::rgb(0.92069976, 0.89140719, 0.79880547);
pub const SAND_300: Srgba = Srgba::rgb(0.86230546, 0.81550779, 0.66888914);
pub const SAND_400: Srgba = Srgba::rgb(0.72154594, 0.65925848, 0.46451492);
pub const SAND_500: Srgba = Srgba::rgb(0.57659849, 0.50691659, 0.28549449);
pub const SAND_600: Srgba = Srgba::rgb(0.46022526, 0.39270136, 0.17166664);
pub const SAND_700: Srgba = Srgba::rgb(0.36840330, 0.31000582, 0.11585930);
pub const SAND_800: Srgba = Srgba::rgb(0.26780613, 0.22116738, 0.06316418);
pub const SAND_900: Srgba = Srgba::rgb(0.20084576, 0.16383970, 0.03839989);
pub const SAND_950: Srgba = Srgba::rgb(0.10560669, 0.08113003, 0.00883781);
// khaki -- #bdb76b at 400, hue 104.5, chroma x0.60 (off by 0.026)
pub const KHAKI_50: Srgba = Srgba::rgb(0.97795528, 0.97655041, 0.94522849);
pub const KHAKI_100: Srgba = Srgba::rgb(0.95376067, 0.95013775, 0.87786337);
pub const KHAKI_200: Srgba = Srgba::rgb(0.90422805, 0.89624021, 0.75980831);
pub const KHAKI_300: Srgba = Srgba::rgb(0.83737066, 0.82231520, 0.60442229);
pub const KHAKI_400: Srgba = Srgba::rgb(0.70955375, 0.68584195, 0.38850904);
pub const KHAKI_500: Srgba = Srgba::rgb(0.58057282, 0.55048597, 0.19196532);
pub const KHAKI_600: Srgba = Srgba::rgb(0.47005714, 0.43881507, 0.02584615);
pub const KHAKI_700: Srgba = Srgba::rgb(0.37681457, 0.35043190, 0.00000000);
pub const KHAKI_800: Srgba = Srgba::rgb(0.28362693, 0.26293774, 0.00000000);
pub const KHAKI_900: Srgba = Srgba::rgb(0.22121980, 0.20434349, 0.00000000);
pub const KHAKI_950: Srgba = Srgba::rgb(0.11942523, 0.10876820, 0.00000000);
// olive -- #808000 at 600, hue 109.8, chroma x0.68 (off by 0.034)
pub const OLIVE_50: Srgba = Srgba::rgb(0.97467737, 0.97710273, 0.94146017);
pub const OLIVE_100: Srgba = Srgba::rgb(0.94622111, 0.95127529, 0.86900747);
pub const OLIVE_200: Srgba = Srgba::rgb(0.89195753, 0.89981512, 0.74429383);
pub const OLIVE_300: Srgba = Srgba::rgb(0.81857637, 0.82750621, 0.57816163);
pub const OLIVE_400: Srgba = Srgba::rgb(0.69455928, 0.70092844, 0.35601097);
pub const OLIVE_500: Srgba = Srgba::rgb(0.57204336, 0.57396834, 0.14007625);
pub const OLIVE_600: Srgba = Srgba::rgb(0.46299621, 0.46299618, 0.00000000);
pub const OLIVE_700: Srgba = Srgba::rgb(0.37131349, 0.37131345, 0.00000000);
pub const OLIVE_800: Srgba = Srgba::rgb(0.28418759, 0.28418755, 0.00000000);
pub const OLIVE_900: Srgba = Srgba::rgb(0.22495551, 0.22495547, 0.00000000);
pub const OLIVE_950: Srgba = Srgba::rgb(0.12270940, 0.12270934, 0.00000000);
// gold -- #ffd700 at 200, hue 95.3, chroma x2.36 (off by 0.036)
pub const GOLD_50: Srgba = Srgba::rgb(1.00000000, 0.97235371, 0.86279835);
pub const GOLD_100: Srgba = Srgba::rgb(1.00000000, 0.93736527, 0.69529757);
pub const GOLD_200: Srgba = Srgba::rgb(1.00000000, 0.87259619, 0.35385789);
pub const GOLD_300: Srgba = Srgba::rgb(0.93154347, 0.78485915, 0.00000000);
pub const GOLD_400: Srgba = Srgba::rgb(0.81666573, 0.68706202, 0.00000000);
pub const GOLD_500: Srgba = Srgba::rgb(0.70567385, 0.59257296, 0.00000000);
pub const GOLD_600: Srgba = Srgba::rgb(0.58940480, 0.49359137, 0.00000000);
pub const GOLD_700: Srgba = Srgba::rgb(0.47981309, 0.40029430, 0.00000000);
pub const GOLD_800: Srgba = Srgba::rgb(0.39200100, 0.32553854, 0.00000000);
pub const GOLD_900: Srgba = Srgba::rgb(0.32834834, 0.27135006, 0.00000000);
pub const GOLD_950: Srgba = Srgba::rgb(0.19238801, 0.15560492, 0.00000000);
// bronze -- #cd7f32 at 500, hue 61.3, chroma x0.70 (off by 0.022)
pub const BRONZE_50: Srgba = Srgba::rgb(0.99917877, 0.96701499, 0.94136944);
pub const BRONZE_100: Srgba = Srgba::rgb(1.00000000, 0.92829644, 0.87066286);
pub const BRONZE_200: Srgba = Srgba::rgb(0.99232830, 0.85610858, 0.74487778);
pub const BRONZE_300: Srgba = Srgba::rgb(0.96918886, 0.75771046, 0.58015080);
pub const BRONZE_400: Srgba = Srgba::rgb(0.88437431, 0.60932378, 0.36420956);
pub const BRONZE_500: Srgba = Srgba::rgb(0.77484029, 0.47127642, 0.16321060);
pub const BRONZE_600: Srgba = Srgba::rgb(0.65054670, 0.36564466, 0.00000000);
pub const BRONZE_700: Srgba = Srgba::rgb(0.52604620, 0.29141779, 0.00000000);
pub const BRONZE_800: Srgba = Srgba::rgb(0.40910557, 0.22169807, 0.00000000);
pub const BRONZE_900: Srgba = Srgba::rgb(0.32926172, 0.17409532, 0.00000000);
pub const BRONZE_950: Srgba = Srgba::rgb(0.18925701, 0.09062463, 0.00000000);
// copper -- #b87333 at 500, hue 60.4, chroma x0.62 (off by 0.016)
pub const COPPER_50: Srgba = Srgba::rgb(0.99738727, 0.96829912, 0.94584197);
pub const COPPER_100: Srgba = Srgba::rgb(0.99749351, 0.93104316, 0.87935597);
pub const COPPER_200: Srgba = Srgba::rgb(0.98393913, 0.86069849, 0.76354234);
pub const COPPER_300: Srgba = Srgba::rgb(0.95689231, 0.76563901, 0.61140210);
pub const COPPER_400: Srgba = Srgba::rgb(0.86063593, 0.61280479, 0.40383901);
pub const COPPER_500: Srgba = Srgba::rgb(0.74202139, 0.46983470, 0.22064312);
pub const COPPER_600: Srgba = Srgba::rgb(0.62137870, 0.36020437, 0.08937029);
pub const COPPER_700: Srgba = Srgba::rgb(0.50737159, 0.28213804, 0.02798847);
pub const COPPER_800: Srgba = Srgba::rgb(0.39002900, 0.20960635, 0.00818125);
pub const COPPER_900: Srgba = Srgba::rgb(0.30716724, 0.16322833, 0.01276631);
pub const COPPER_950: Srgba = Srgba::rgb(0.17655706, 0.08151517, 0.00009092);
// clay -- #b66a50 at 500, hue 39.6, chroma x0.56 (off by 0.022)
pub const CLAY_50: Srgba = Srgba::rgb(1.00000000, 0.96684899, 0.95573091);
pub const CLAY_100: Srgba = Srgba::rgb(1.00000000, 0.92889273, 0.90496737);
pub const CLAY_200: Srgba = Srgba::rgb(0.99298613, 0.85350016, 0.80634311);
pub const CLAY_300: Srgba = Srgba::rgb(0.96978875, 0.75473139, 0.68175823);
pub const CLAY_400: Srgba = Srgba::rgb(0.86824083, 0.59213711, 0.49819904);
pub const CLAY_500: Srgba = Srgba::rgb(0.74244632, 0.44169024, 0.33891670);
pub const CLAY_600: Srgba = Srgba::rgb(0.61870951, 0.33137566, 0.23244382);
pub const CLAY_700: Srgba = Srgba::rgb(0.50444957, 0.25695113, 0.17138709);
pub const CLAY_800: Srgba = Srgba::rgb(0.38363114, 0.18584577, 0.11728663);
pub const CLAY_900: Srgba = Srgba::rgb(0.29914393, 0.14156992, 0.08705297);
pub const CLAY_950: Srgba = Srgba::rgb(0.17073185, 0.06682520, 0.03122030);
// terracotta -- #e2725b at 500, hue 32.9, chroma x0.77 (off by 0.025)
pub const TERRACOTTA_50: Srgba = Srgba::rgb(1.00000000, 0.96366558, 0.95535475);
pub const TERRACOTTA_100: Srgba = Srgba::rgb(1.00000000, 0.92132213, 0.90334359);
pub const TERRACOTTA_200: Srgba = Srgba::rgb(1.00000000, 0.84198784, 0.80608080);
pub const TERRACOTTA_300: Srgba = Srgba::rgb(1.00000000, 0.72643162, 0.66544610);
pub const TERRACOTTA_400: Srgba = Srgba::rgb(0.95321067, 0.55906853, 0.47508115);
pub const TERRACOTTA_500: Srgba = Srgba::rgb(0.85271593, 0.41702163, 0.32817978);
pub const TERRACOTTA_600: Srgba = Srgba::rgb(0.72956859, 0.30850826, 0.22517521);
pub const TERRACOTTA_700: Srgba = Srgba::rgb(0.60073575, 0.23655193, 0.16512096);
pub const TERRACOTTA_800: Srgba = Srgba::rgb(0.47404170, 0.18118560, 0.12369667);
pub const TERRACOTTA_900: Srgba = Srgba::rgb(0.38160098, 0.14764288, 0.10132074);
pub const TERRACOTTA_950: Srgba = Srgba::rgb(0.22644538, 0.07151038, 0.04105246);
// brown -- #8b5a2b at 600, hue 62.2, chroma x0.48 (off by 0.005)
pub const BROWN_50: Srgba = Srgba::rgb(0.99296307, 0.97123004, 0.95337192);
pub const BROWN_100: Srgba = Srgba::rgb(0.98777868, 0.93804335, 0.89694759);
pub const BROWN_200: Srgba = Srgba::rgb(0.96381116, 0.87142030, 0.79432125);
pub const BROWN_300: Srgba = Srgba::rgb(0.92718155, 0.78359944, 0.66188930);
pub const BROWN_400: Srgba = Srgba::rgb(0.80828151, 0.62305879, 0.46160495);
pub const BROWN_500: Srgba = Srgba::rgb(0.67293548, 0.47106881, 0.28726630);
pub const BROWN_600: Srgba = Srgba::rgb(0.55169371, 0.35905791, 0.17505616);
pub const BROWN_700: Srgba = Srgba::rgb(0.44681575, 0.28099120, 0.11912433);
pub const BROWN_800: Srgba = Srgba::rgb(0.33320251, 0.20107581, 0.06951463);
pub const BROWN_900: Srgba = Srgba::rgb(0.25538266, 0.15035375, 0.04621458);
pub const BROWN_950: Srgba = Srgba::rgb(0.14159102, 0.07242445, 0.01154491);
// sepia -- #704214 at 700, hue 61.1, chroma x0.52 (off by 0.020)
pub const SEPIA_50: Srgba = Srgba::rgb(0.99424869, 0.97040643, 0.95157145);
pub const SEPIA_100: Srgba = Srgba::rgb(0.99061584, 0.93608649, 0.89274551);
pub const SEPIA_200: Srgba = Srgba::rgb(0.96960406, 0.86837546, 0.78704310);
pub const SEPIA_300: Srgba = Srgba::rgb(0.93576758, 0.77856138, 0.65005219);
pub const SEPIA_400: Srgba = Srgba::rgb(0.82283768, 0.61989159, 0.44873194);
pub const SEPIA_500: Srgba = Srgba::rgb(0.69172751, 0.47018008, 0.27370600);
pub const SEPIA_600: Srgba = Srgba::rgb(0.57050210, 0.35882186, 0.15988536);
pub const SEPIA_700: Srgba = Srgba::rgb(0.46313040, 0.28083562, 0.10472855);
pub const SEPIA_800: Srgba = Srgba::rgb(0.34831535, 0.20285276, 0.05918335);
pub const SEPIA_900: Srgba = Srgba::rgb(0.26903299, 0.15328057, 0.03998182);
pub const SEPIA_950: Srgba = Srgba::rgb(0.15078355, 0.07450220, 0.00933878);
// taupe -- #483c32 at 700, hue 61.8, chroma x0.14 (off by 0.027)
pub const TAUPE_50: Srgba = Srgba::rgb(0.98384093, 0.97737426, 0.97215327);
pub const TAUPE_100: Srgba = Srgba::rgb(0.96747726, 0.95263084, 0.94062484);
pub const TAUPE_200: Srgba = Srgba::rgb(0.92020570, 0.89253998, 0.87010521);
pub const TAUPE_300: Srgba = Srgba::rgb(0.86125308, 0.81809682, 0.78296382);
pub const TAUPE_400: Srgba = Srgba::rgb(0.68960078, 0.63437482, 0.58914498);
pub const TAUPE_500: Srgba = Srgba::rgb(0.51666502, 0.45741375, 0.40853536);
pub const TAUPE_600: Srgba = Srgba::rgb(0.39475852, 0.33887053, 0.29250723);
pub const TAUPE_700: Srgba = Srgba::rgb(0.31063883, 0.26271330, 0.22287675);
pub const TAUPE_800: Srgba = Srgba::rgb(0.20709893, 0.16957032, 0.13826045);
pub const TAUPE_900: Srgba = Srgba::rgb(0.14167875, 0.11227450, 0.08769300);
pub const TAUPE_950: Srgba = Srgba::rgb(0.06516812, 0.04596926, 0.03050642);
// peach -- #ffcba4 at 300, hue 58.4, chroma x0.64 (off by 0.027)
pub const PEACH_50: Srgba = Srgba::rgb(0.99848910, 0.96764869, 0.94548193);
pub const PEACH_100: Srgba = Srgba::rgb(0.99991503, 0.92951250, 0.87851461);
pub const PEACH_200: Srgba = Srgba::rgb(0.98853458, 0.85808678, 0.76228349);
pub const PEACH_300: Srgba = Srgba::rgb(0.96358740, 0.76135187, 0.60937689);
pub const PEACH_400: Srgba = Srgba::rgb(0.87058346, 0.60858418, 0.40271578);
pub const PEACH_500: Srgba = Srgba::rgb(0.75402706, 0.46615326, 0.22048029);
pub const PEACH_600: Srgba = Srgba::rgb(0.63314597, 0.35678403, 0.08928531);
pub const PEACH_700: Srgba = Srgba::rgb(0.51753309, 0.27915599, 0.02774973);
pub const PEACH_800: Srgba = Srgba::rgb(0.39915956, 0.20810852, 0.00917394);
pub const PEACH_900: Srgba = Srgba::rgb(0.31524418, 0.16277466, 0.01420872);
pub const PEACH_950: Srgba = Srgba::rgb(0.18196988, 0.08126899, 0.00055426);
// coral -- #ff7f50 at 400, hue 40.2, chroma x1.02 (off by 0.033)
pub const CORAL_50: Srgba = Srgba::rgb(1.00000000, 0.96119342, 0.94772465);
pub const CORAL_100: Srgba = Srgba::rgb(1.00000000, 0.91521312, 0.88566300);
pub const CORAL_200: Srgba = Srgba::rgb(1.00000000, 0.83654957, 0.77925487);
pub const CORAL_300: Srgba = Srgba::rgb(1.00000000, 0.71729236, 0.61758565);
pub const CORAL_400: Srgba = Srgba::rgb(1.00000000, 0.55732460, 0.39848919);
pub const CORAL_500: Srgba = Srgba::rgb(0.96592793, 0.39538443, 0.16259083);
pub const CORAL_600: Srgba = Srgba::rgb(0.83937817, 0.29317813, 0.00000000);
pub const CORAL_700: Srgba = Srgba::rgb(0.68727439, 0.23396457, 0.00000000);
pub const CORAL_800: Srgba = Srgba::rgb(0.56539884, 0.18651874, 0.00000000);
pub const CORAL_900: Srgba = Srgba::rgb(0.47131588, 0.15902683, 0.01806415);
pub const CORAL_950: Srgba = Srgba::rgb(0.28738421, 0.07985866, 0.00163865);
// salmon -- #fa8072 at 400, hue 28.1, chroma x0.92 (off by 0.023)
pub const SALMON_50: Srgba = Srgba::rgb(1.00000000, 0.96134421, 0.95535697);
pub const SALMON_100: Srgba = Srgba::rgb(1.00000000, 0.91579072, 0.90285807);
pub const SALMON_200: Srgba = Srgba::rgb(1.00000000, 0.83544734, 0.81073185);
pub const SALMON_300: Srgba = Srgba::rgb(1.00000000, 0.71531404, 0.67493874);
pub const SALMON_400: Srgba = Srgba::rgb(1.00000000, 0.53843071, 0.48335588);
pub const SALMON_500: Srgba = Srgba::rgb(0.93001700, 0.39061871, 0.33885273);
pub const SALMON_600: Srgba = Srgba::rgb(0.80799666, 0.28122571, 0.23951504);
pub const SALMON_700: Srgba = Srgba::rgb(0.66901116, 0.21153654, 0.17821369);
pub const SALMON_800: Srgba = Srgba::rgb(0.53912599, 0.17095151, 0.14279553);
pub const SALMON_900: Srgba = Srgba::rgb(0.44155063, 0.14782384, 0.12300433);
pub const SALMON_950: Srgba = Srgba::rgb(0.26708798, 0.07206266, 0.05634671);
// blush -- #de5d83 at 500, hue 4.5, chroma x0.88 (off by 0.015)
pub const BLUSH_50: Srgba = Srgba::rgb(1.00000000, 0.96026537, 0.96764413);
pub const BLUSH_100: Srgba = Srgba::rgb(1.00000000, 0.91353068, 0.93007642);
pub const BLUSH_200: Srgba = Srgba::rgb(1.00000000, 0.82947585, 0.86406406);
pub const BLUSH_300: Srgba = Srgba::rgb(1.00000000, 0.70419573, 0.77039934);
pub const BLUSH_400: Srgba = Srgba::rgb(0.98047033, 0.52448516, 0.64379967);
pub const BLUSH_500: Srgba = Srgba::rgb(0.89174844, 0.38374286, 0.53166582);
pub const BLUSH_600: Srgba = Srgba::rgb(0.76990275, 0.27553854, 0.43056136);
pub const BLUSH_700: Srgba = Srgba::rgb(0.63594716, 0.20715325, 0.34512617);
pub const BLUSH_800: Srgba = Srgba::rgb(0.50943084, 0.16429036, 0.27412467);
pub const BLUSH_900: Srgba = Srgba::rgb(0.41539193, 0.13985039, 0.22486342);
pub const BLUSH_950: Srgba = Srgba::rgb(0.24948102, 0.06666202, 0.12406841);
// crimson -- #dc143c at 600, hue 20.1, chroma x1.19 (off by 0.027)
pub const CRIMSON_50: Srgba = Srgba::rgb(1.00000000, 0.95971466, 0.95814002);
pub const CRIMSON_100: Srgba = Srgba::rgb(1.00000000, 0.91197087, 0.90879714);
pub const CRIMSON_200: Srgba = Srgba::rgb(1.00000000, 0.83026478, 0.82524588);
pub const CRIMSON_300: Srgba = Srgba::rgb(1.00000000, 0.70632511, 0.70172424);
pub const CRIMSON_400: Srgba = Srgba::rgb(1.00000000, 0.53968768, 0.54621213);
pub const CRIMSON_500: Srgba = Srgba::rgb(1.00000000, 0.32721049, 0.38132353);
pub const CRIMSON_600: Srgba = Srgba::rgb(0.90110963, 0.14511234, 0.26558808);
pub const CRIMSON_700: Srgba = Srgba::rgb(0.74927770, 0.07159769, 0.20175147);
pub const CRIMSON_800: Srgba = Srgba::rgb(0.60987912, 0.07861070, 0.16596689);
pub const CRIMSON_900: Srgba = Srgba::rgb(0.50319330, 0.09134895, 0.14449091);
pub const CRIMSON_950: Srgba = Srgba::rgb(0.30831606, 0.03250321, 0.07120478);
// wine -- #722f37 at 800, hue 15.1, chroma x0.67 (off by 0.011)
pub const WINE_50: Srgba = Srgba::rgb(1.00000000, 0.96375140, 0.96490375);
pub const WINE_100: Srgba = Srgba::rgb(1.00000000, 0.92180627, 0.92456009);
pub const WINE_200: Srgba = Srgba::rgb(1.00000000, 0.83995199, 0.84685809);
pub const WINE_300: Srgba = Srgba::rgb(1.00000000, 0.72249807, 0.73864125);
pub const WINE_400: Srgba = Srgba::rgb(0.91774322, 0.55947979, 0.58730284);
pub const WINE_500: Srgba = Srgba::rgb(0.80515806, 0.41240354, 0.45137577);
pub const WINE_600: Srgba = Srgba::rgb(0.68111102, 0.30372924, 0.34747951);
pub const WINE_700: Srgba = Srgba::rgb(0.55849641, 0.23272632, 0.27245737);
pub const WINE_800: Srgba = Srgba::rgb(0.43406348, 0.17267163, 0.20485781);
pub const WINE_900: Srgba = Srgba::rgb(0.34500348, 0.13632765, 0.16113071);
pub const WINE_950: Srgba = Srgba::rgb(0.20163999, 0.06368675, 0.08081757);
// chartreuse -- #7fff00 at 200, hue 136.0, chroma x2.60 (off by 0.066)
pub const CHARTREUSE_50: Srgba = Srgba::rgb(0.91964639, 1.00000000, 0.88430223);
pub const CHARTREUSE_100: Srgba = Srgba::rgb(0.81578235, 1.00000000, 0.73119684);
pub const CHARTREUSE_200: Srgba = Srgba::rgb(0.62483889, 0.99194133, 0.42592593);
pub const CHARTREUSE_300: Srgba = Srgba::rgb(0.45961987, 0.92670952, 0.00000000);
pub const CHARTREUSE_400: Srgba = Srgba::rgb(0.39969510, 0.81239467, 0.00000000);
pub const CHARTREUSE_500: Srgba = Srgba::rgb(0.34179733, 0.70194663, 0.00000000);
pub const CHARTREUSE_600: Srgba = Srgba::rgb(0.28114678, 0.58624728, 0.00000000);
pub const CHARTREUSE_700: Srgba = Srgba::rgb(0.22397937, 0.47719255, 0.00000000);
pub const CHARTREUSE_800: Srgba = Srgba::rgb(0.17817305, 0.38981072, 0.00000000);
pub const CHARTREUSE_900: Srgba = Srgba::rgb(0.14496925, 0.32646996, 0.00000000);
pub const CHARTREUSE_950: Srgba = Srgba::rgb(0.07404668, 0.19117579, 0.00000000);
// moss -- #8a9a5b at 500, hue 120.5, chroma x0.47 (off by 0.044)
pub const MOSS_50: Srgba = Srgba::rgb(0.97197172, 0.97937825, 0.95530109);
pub const MOSS_100: Srgba = Srgba::rgb(0.94003068, 0.95684081, 0.90139353);
pub const MOSS_200: Srgba = Srgba::rgb(0.87582714, 0.90658954, 0.80247402);
pub const MOSS_300: Srgba = Srgba::rgb(0.79247783, 0.83920465, 0.67479237);
pub const MOSS_400: Srgba = Srgba::rgb(0.63717678, 0.69540781, 0.47791199);
pub const MOSS_500: Srgba = Srgba::rgb(0.48982787, 0.55090403, 0.30566108);
pub const MOSS_600: Srgba = Srgba::rgb(0.37951940, 0.43619497, 0.19428225);
pub const MOSS_700: Srgba = Srgba::rgb(0.29938096, 0.34770389, 0.13653048);
pub const MOSS_800: Srgba = Srgba::rgb(0.21573693, 0.25392920, 0.08355667);
pub const MOSS_900: Srgba = Srgba::rgb(0.16154599, 0.19193361, 0.05683125);
pub const MOSS_950: Srgba = Srgba::rgb(0.08011356, 0.09991425, 0.01561551);
// sage -- #9caf88 at 400, hue 128.9, chroma x0.36 (off by 0.002)
pub const SAGE_50: Srgba = Srgba::rgb(0.97142237, 0.98026871, 0.96291340);
pub const SAGE_100: Srgba = Srgba::rgb(0.93879086, 0.95904467, 0.91912121);
pub const SAGE_200: Srgba = Srgba::rgb(0.87099816, 0.90860124, 0.83385811);
pub const SAGE_300: Srgba = Srgba::rgb(0.78444822, 0.84277394, 0.72538570);
pub const SAGE_400: Srgba = Srgba::rgb(0.61374964, 0.68830573, 0.53528901);
pub const SAGE_500: Srgba = Srgba::rgb(0.45346417, 0.53362257, 0.36511688);
pub const SAGE_600: Srgba = Srgba::rgb(0.34105473, 0.41675935, 0.25447506);
pub const SAGE_700: Srgba = Srgba::rgb(0.26558213, 0.33051951, 0.19031395);
pub const SAGE_800: Srgba = Srgba::rgb(0.18278289, 0.23410714, 0.12232114);
pub const SAGE_900: Srgba = Srgba::rgb(0.13089612, 0.17149982, 0.08290898);
pub const SAGE_950: Srgba = Srgba::rgb(0.05927457, 0.08587036, 0.02830573);
// forest -- #228b22 at 600, hue 142.9, chroma x0.90 (off by 0.025)
pub const FOREST_50: Srgba = Srgba::rgb(0.94825813, 0.98330712, 0.94483889);
pub const FOREST_100: Srgba = Srgba::rgb(0.88434658, 0.96510270, 0.87680771);
pub const FOREST_200: Srgba = Srgba::rgb(0.77717946, 0.92951153, 0.76414776);
pub const FOREST_300: Srgba = Srgba::rgb(0.62957417, 0.87257778, 0.61191310);
pub const FOREST_400: Srgba = Srgba::rgb(0.45085506, 0.78295880, 0.43305472);
pub const FOREST_500: Srgba = Srgba::rgb(0.29287464, 0.68410655, 0.28055215);
pub const FOREST_600: Srgba = Srgba::rgb(0.17353632, 0.57556284, 0.16925092);
pub const FOREST_700: Srgba = Srgba::rgb(0.11182938, 0.46965706, 0.11148832);
pub const FOREST_800: Srgba = Srgba::rgb(0.09237805, 0.37649684, 0.09061361);
pub const FOREST_900: Srgba = Srgba::rgb(0.08841069, 0.30890201, 0.08464071);
pub const FOREST_950: Srgba = Srgba::rgb(0.03169368, 0.17960822, 0.03018127);
// jade -- #00a86b at 500, hue 158.8, chroma x0.79 (off by 0.011)
pub const JADE_50: Srgba = Srgba::rgb(0.94358280, 0.98475202, 0.95902379);
pub const JADE_100: Srgba = Srgba::rgb(0.87302636, 0.96859949, 0.90984893);
pub const JADE_200: Srgba = Srgba::rgb(0.75127674, 0.93408074, 0.82519465);
pub const JADE_300: Srgba = Srgba::rgb(0.58102626, 0.88024977, 0.71172750);
pub const JADE_400: Srgba = Srgba::rgb(0.35010099, 0.78138689, 0.56292040);
pub const JADE_500: Srgba = Srgba::rgb(0.05548079, 0.67245844, 0.43222628);
pub const JADE_600: Srgba = Srgba::rgb(0.00000000, 0.55110284, 0.34798622);
pub const JADE_700: Srgba = Srgba::rgb(0.00000000, 0.44540860, 0.27771210);
pub const JADE_800: Srgba = Srgba::rgb(0.00000000, 0.35073086, 0.21476263);
pub const JADE_900: Srgba = Srgba::rgb(0.00000000, 0.28494068, 0.17101994);
pub const JADE_950: Srgba = Srgba::rgb(0.00000000, 0.16220405, 0.08941452);
// mint -- #3eb489 at 500, hue 165.1, chroma x0.66 (off by 0.054)
pub const MINT_50: Srgba = Srgba::rgb(0.94752178, 0.98433540, 0.96609079);
pub const MINT_100: Srgba = Srgba::rgb(0.88243267, 0.96788610, 0.92627217);
pub const MINT_200: Srgba = Srgba::rgb(0.76723617, 0.93041206, 0.85350944);
pub const MINT_300: Srgba = Srgba::rgb(0.60960245, 0.87529990, 0.75680816);
pub const MINT_400: Srgba = Srgba::rgb(0.38511258, 0.76101282, 0.60916903);
pub const MINT_500: Srgba = Srgba::rgb(0.15023887, 0.63824475, 0.47362085);
pub const MINT_600: Srgba = Srgba::rgb(0.00000000, 0.51943494, 0.37322712);
pub const MINT_700: Srgba = Srgba::rgb(0.00000000, 0.41748931, 0.29722912);
pub const MINT_800: Srgba = Srgba::rgb(0.00000000, 0.31961805, 0.22426847);
pub const MINT_900: Srgba = Srgba::rgb(0.00000000, 0.25332974, 0.17485212);
pub const MINT_950: Srgba = Srgba::rgb(0.00000000, 0.14047806, 0.09072383);
// seafoam -- #93e9be at 300, hue 161.5, chroma x0.84 (off by 0.023)
pub const SEAFOAM_50: Srgba = Srgba::rgb(0.93981334, 0.98528795, 0.95953502);
pub const SEAFOAM_100: Srgba = Srgba::rgb(0.86394082, 0.96971381, 0.91100135);
pub const SEAFOAM_200: Srgba = Srgba::rgb(0.73372417, 0.93700612, 0.82845365);
pub const SEAFOAM_300: Srgba = Srgba::rgb(0.54781208, 0.88440030, 0.71698426);
pub const SEAFOAM_400: Srgba = Srgba::rgb(0.28823800, 0.79209274, 0.57563583);
pub const SEAFOAM_500: Srgba = Srgba::rgb(0.00000000, 0.68129705, 0.46159216);
pub const SEAFOAM_600: Srgba = Srgba::rgb(0.00000000, 0.56018993, 0.37662237);
pub const SEAFOAM_700: Srgba = Srgba::rgb(0.00000000, 0.45360946, 0.30184459);
pub const SEAFOAM_800: Srgba = Srgba::rgb(0.00000000, 0.36072932, 0.23667907);
pub const SEAFOAM_900: Srgba = Srgba::rgb(0.00000000, 0.29552020, 0.19092777);
pub const SEAFOAM_950: Srgba = Srgba::rgb(0.00000000, 0.16957311, 0.10256210);
// aqua -- #00ffff at 200, hue 194.8, chroma x2.00 (off by 0.000)
pub const AQUA_50: Srgba = Srgba::rgb(0.87596481, 1.00000000, 0.99687122);
pub const AQUA_100: Srgba = Srgba::rgb(0.70257115, 1.00000000, 0.99545420);
pub const AQUA_200: Srgba = Srgba::rgb(0.02488782, 1.00000000, 0.99995715);
pub const AQUA_300: Srgba = Srgba::rgb(0.00000000, 0.90623698, 0.90623690);
pub const AQUA_400: Srgba = Srgba::rgb(0.00000000, 0.79430604, 0.79430597);
pub const AQUA_500: Srgba = Srgba::rgb(0.00000000, 0.68616129, 0.68616122);
pub const AQUA_600: Srgba = Srgba::rgb(0.00000000, 0.57287472, 0.57287467);
pub const AQUA_700: Srgba = Srgba::rgb(0.00000000, 0.46609422, 0.46609418);
pub const AQUA_800: Srgba = Srgba::rgb(0.00000000, 0.38053464, 0.38053461);
pub const AQUA_900: Srgba = Srgba::rgb(0.00000000, 0.31851478, 0.31851475);
pub const AQUA_950: Srgba = Srgba::rgb(0.00000000, 0.18604202, 0.18604201);
// mist -- #c6d3dd at 300, hue 240.8, chroma x0.16 (off by 0.005)
pub const MIST_50: Srgba = Srgba::rgb(0.97215411, 0.97970011, 0.98556252);
pub const MIST_100: Srgba = Srgba::rgb(0.94058435, 0.95795428, 0.97142313);
pub const MIST_200: Srgba = Srgba::rgb(0.87035099, 0.90286874, 0.92800094);
pub const MIST_300: Srgba = Srgba::rgb(0.78315254, 0.83420797, 0.87348413);
pub const MIST_400: Srgba = Srgba::rgb(0.59112955, 0.65716728, 0.70760127);
pub const MIST_500: Srgba = Srgba::rgb(0.41185740, 0.48364149, 0.53798006);
pub const MIST_600: Srgba = Srgba::rgb(0.29580216, 0.36419351, 0.41560289);
pub const MIST_700: Srgba = Srgba::rgb(0.22567582, 0.28452848, 0.32865766);
pub const MIST_800: Srgba = Srgba::rgb(0.14129904, 0.18771212, 0.22235707);
pub const MIST_900: Srgba = Srgba::rgb(0.09076215, 0.12727964, 0.15447346);
pub const MIST_950: Srgba = Srgba::rgb(0.03228261, 0.05587776, 0.07366531);
// steel -- #4682b4 at 500, hue 245.7, chroma x0.53 (off by 0.034)
pub const STEEL_50: Srgba = Srgba::rgb(0.95587474, 0.97837814, 0.99917650);
pub const STEEL_100: Srgba = Srgba::rgb(0.90416219, 0.95408959, 1.00000000);
pub const STEEL_200: Srgba = Srgba::rgb(0.80515938, 0.90278524, 0.99168428);
pub const STEEL_300: Srgba = Srgba::rgb(0.67764385, 0.83239671, 0.97125533);
pub const STEEL_400: Srgba = Srgba::rgb(0.48488874, 0.69136208, 0.87198576);
pub const STEEL_500: Srgba = Srgba::rgb(0.31436514, 0.54997206, 0.74847093);
pub const STEEL_600: Srgba = Srgba::rgb(0.20090181, 0.43586388, 0.62623356);
pub const STEEL_700: Srgba = Srgba::rgb(0.14133490, 0.34737341, 0.51149172);
pub const STEEL_800: Srgba = Srgba::rgb(0.09011680, 0.25671000, 0.38780183);
pub const STEEL_900: Srgba = Srgba::rgb(0.06484004, 0.19664119, 0.30096786);
pub const STEEL_950: Srgba = Srgba::rgb(0.01880829, 0.10320707, 0.17203022);
// azure -- #007fff at 600, hue 256.3, chroma x1.13 (off by 0.016)
pub const AZURE_50: Srgba = Srgba::rgb(0.95243581, 0.97242784, 1.00000000);
pub const AZURE_100: Srgba = Srgba::rgb(0.89647068, 0.94003446, 1.00000000);
pub const AZURE_200: Srgba = Srgba::rgb(0.80196842, 0.88542704, 1.00000000);
pub const AZURE_300: Srgba = Srgba::rgb(0.66280431, 0.80517326, 1.00000000);
pub const AZURE_400: Srgba = Srgba::rgb(0.48765696, 0.70480196, 1.00000000);
pub const AZURE_500: Srgba = Srgba::rgb(0.29614924, 0.59968526, 1.00000000);
pub const AZURE_600: Srgba = Srgba::rgb(0.00000000, 0.48083176, 0.96717467);
pub const AZURE_700: Srgba = Srgba::rgb(0.00000000, 0.38970466, 0.79333701);
pub const AZURE_800: Srgba = Srgba::rgb(0.00000000, 0.31668761, 0.65404685);
pub const AZURE_900: Srgba = Srgba::rgb(0.01661229, 0.26526514, 0.54520374);
pub const AZURE_950: Srgba = Srgba::rgb(0.00087174, 0.15085758, 0.33664309);
// periwinkle -- #ccccff at 300, hue 284.8, chroma x0.58 (off by 0.009)
pub const PERIWINKLE_50: Srgba = Srgba::rgb(0.97091330, 0.97176914, 1.00000000);
pub const PERIWINKLE_100: Srgba = Srgba::rgb(0.93807542, 0.93962524, 1.00000000);
pub const PERIWINKLE_200: Srgba = Srgba::rgb(0.87311086, 0.87498487, 1.00000000);
pub const PERIWINKLE_300: Srgba = Srgba::rgb(0.78847073, 0.78839193, 0.98785261);
pub const PERIWINKLE_400: Srgba = Srgba::rgb(0.64449764, 0.63891829, 0.89792414);
pub const PERIWINKLE_500: Srgba = Srgba::rgb(0.50852152, 0.49609981, 0.78067445);
pub const PERIWINKLE_600: Srgba = Srgba::rgb(0.40141312, 0.38521451, 0.65821148);
pub const PERIWINKLE_700: Srgba = Srgba::rgb(0.31904050, 0.30382137, 0.53921118);
pub const PERIWINKLE_800: Srgba = Srgba::rgb(0.23707285, 0.22440416, 0.41269103);
pub const PERIWINKLE_900: Srgba = Srgba::rgb(0.18263514, 0.17288341, 0.32291005);
pub const PERIWINKLE_950: Srgba = Srgba::rgb(0.09469584, 0.08774758, 0.18676496);
// lavender -- #b57edc at 500, hue 310.0, chroma x0.77 (off by 0.031)
pub const LAVENDER_50: Srgba = Srgba::rgb(0.98342215, 0.96380770, 1.00000000);
pub const LAVENDER_100: Srgba = Srgba::rgb(0.96448131, 0.92178029, 1.00000000);
pub const LAVENDER_200: Srgba = Srgba::rgb(0.93022939, 0.84369147, 1.00000000);
pub const LAVENDER_300: Srgba = Srgba::rgb(0.87783851, 0.73731129, 0.98651531);
pub const LAVENDER_400: Srgba = Srgba::rgb(0.77917381, 0.59042228, 0.91806379);
pub const LAVENDER_500: Srgba = Srgba::rgb(0.67101726, 0.45645722, 0.82178448);
pub const LAVENDER_600: Srgba = Srgba::rgb(0.56078527, 0.35020355, 0.70414994);
pub const LAVENDER_700: Srgba = Srgba::rgb(0.45672561, 0.27376467, 0.57995144);
pub const LAVENDER_800: Srgba = Srgba::rgb(0.35824616, 0.21117911, 0.45741108);
pub const LAVENDER_900: Srgba = Srgba::rgb(0.28802830, 0.17103909, 0.36772565);
pub const LAVENDER_950: Srgba = Srgba::rgb(0.16514660, 0.08739539, 0.21767142);
// lilac -- #c8a2c8 at 400, hue 326.2, chroma x0.41 (off by 0.025)
pub const LILAC_50: Srgba = Srgba::rgb(0.98706145, 0.96982816, 0.98670963);
pub const LILAC_100: Srgba = Srgba::rgb(0.97447785, 0.93497749, 0.97378118);
pub const LILAC_200: Srgba = Srgba::rgb(0.93822101, 0.86469672, 0.93725832);
pub const LILAC_300: Srgba = Srgba::rgb(0.88832404, 0.77380033, 0.88751452);
pub const LILAC_400: Srgba = Srgba::rgb(0.75234321, 0.60459282, 0.75244363);
pub const LILAC_500: Srgba = Srgba::rgb(0.60736139, 0.44656292, 0.60864852);
pub const LILAC_600: Srgba = Srgba::rgb(0.48799850, 0.33470629, 0.48994358);
pub const LILAC_700: Srgba = Srgba::rgb(0.39189282, 0.25997416, 0.39376330);
pub const LILAC_800: Srgba = Srgba::rgb(0.28586636, 0.18100473, 0.28750211);
pub const LILAC_900: Srgba = Srgba::rgb(0.21491995, 0.13174130, 0.21622244);
pub const LILAC_950: Srgba = Srgba::rgb(0.11465995, 0.05992399, 0.11560596);
// plum -- #8e4585 at 700, hue 332.1, chroma x0.77 (off by 0.023)
pub const PLUM_50: Srgba = Srgba::rgb(0.99610469, 0.96032082, 0.98903661);
pub const PLUM_100: Srgba = Srgba::rgb(0.99419551, 0.91255626, 0.97867059);
pub const PLUM_200: Srgba = Srgba::rgb(0.98013296, 0.82887088, 0.95313586);
pub const PLUM_300: Srgba = Srgba::rgb(0.94971856, 0.71509648, 0.91140620);
pub const PLUM_400: Srgba = Srgba::rgb(0.86657048, 0.56058992, 0.82185215);
pub const PLUM_500: Srgba = Srgba::rgb(0.76184966, 0.42231164, 0.71706825);
pub const PLUM_600: Srgba = Srgba::rgb(0.64480764, 0.31632834, 0.60439045);
pub const PLUM_700: Srgba = Srgba::rgb(0.52830370, 0.24418043, 0.49415199);
pub const PLUM_800: Srgba = Srgba::rgb(0.41587968, 0.18739202, 0.38834769);
pub const PLUM_900: Srgba = Srgba::rgb(0.33472566, 0.15219145, 0.31223595);
pub const PLUM_950: Srgba = Srgba::rgb(0.19570678, 0.07481879, 0.18108130);
// mauve -- #e0b0ff at 300, hue 311.6, chroma x0.97 (off by 0.017)
pub const MAUVE_50: Srgba = Srgba::rgb(0.98360277, 0.96065699, 1.00000000);
pub const MAUVE_100: Srgba = Srgba::rgb(0.96460827, 0.91429865, 1.00000000);
pub const MAUVE_200: Srgba = Srgba::rgb(0.93286210, 0.83468358, 1.00000000);
pub const MAUVE_300: Srgba = Srgba::rgb(0.88793505, 0.71631584, 1.00000000);
pub const MAUVE_400: Srgba = Srgba::rgb(0.82462862, 0.56837958, 0.98183976);
pub const MAUVE_500: Srgba = Srgba::rgb(0.73924836, 0.44450527, 0.91157188);
pub const MAUVE_600: Srgba = Srgba::rgb(0.63226840, 0.34061050, 0.79708541);
pub const MAUVE_700: Srgba = Srgba::rgb(0.51935174, 0.26521537, 0.66125894);
pub const MAUVE_800: Srgba = Srgba::rgb(0.42056557, 0.21641681, 0.53556034);
pub const MAUVE_900: Srgba = Srgba::rgb(0.34710381, 0.18499115, 0.43998202);
pub const MAUVE_950: Srgba = Srgba::rgb(0.20543767, 0.09753615, 0.26681425);
