//! Palette families beyond the Tailwind set, on the same eleven-step scale.
//!
//! [`bevy_color::palettes::tailwind`] carries seventeen chromatic hues and five neutrals. This
//! adds the ones a UI reaches for and finds missing: warm earths (`clay`, `terracotta`, `sepia`,
//! `taupe`), metals (`gold`, `bronze`, `copper`), the yellow-green arc Tailwind jumps over
//! between `yellow` and `lime` (`olive`, `chartreuse`), muted greens (`sage`, `moss`, `forest`),
//! quiet tinted greys (`mist`, `steel`), and the purple-to-red stretch (`lilac`, `plum`, `mauve`,
//! `wine`, `crimson`). Reached the same way as any other family -- [`Color::olive`](crate::Color).
//!
//! # How these were built
//!
//! Not picked by eye. Every step is placed in OKLCH against a curve read back out of Tailwind
//! itself, so a step here means what the same step means there:
//!
//! - **Lightness** comes from Tailwind's own two curves. Its chromatic families and its neutrals
//!   do not share one -- `stone(700)` is L 0.372 and the average chromatic 700 is L 0.515 -- so a
//!   family's curve interpolates between them by how chromatic it is. A near-grey like `mist`
//!   lands on the neutral curve where it will be compared against `zinc`; a saturated one like
//!   `crimson` lands on the chromatic curve beside `red`.
//! - **Chroma** follows Tailwind's own arc over the ramp (rising to a peak at 500, easing off
//!   toward both ends), scaled per family.
//! - **Hue** is fixed per family and held across all eleven steps, so a family does not drift
//!   toward another one at its dark end.
//! - Any step whose target falls outside sRGB has its chroma backed off until it fits, which is
//!   what keeps the light end from flattening into the same near-white for every family.

use bevy_color::Srgba;

// sand
pub const SAND_50: Srgba = Srgba::rgb(0.98505923, 0.97577822, 0.96079794);
pub const SAND_100: Srgba = Srgba::rgb(0.97010783, 0.94875146, 0.91425456);
pub const SAND_200: Srgba = Srgba::rgb(0.92863406, 0.88865208, 0.82394659);
pub const SAND_300: Srgba = Srgba::rgb(0.87427242, 0.81153284, 0.70957218);
pub const SAND_400: Srgba = Srgba::rgb(0.72544200, 0.64402803, 0.51040161);
pub const SAND_500: Srgba = Srgba::rgb(0.57108463, 0.48222016, 0.33374028);
pub const SAND_600: Srgba = Srgba::rgb(0.45150229, 0.36669847, 0.22226159);
pub const SAND_700: Srgba = Srgba::rgb(0.36025227, 0.28725896, 0.16192873);
pub const SAND_800: Srgba = Srgba::rgb(0.25576618, 0.19791233, 0.09741019);
pub const SAND_900: Srgba = Srgba::rgb(0.18719131, 0.14144028, 0.06169005);
pub const SAND_950: Srgba = Srgba::rgb(0.09599016, 0.06590744, 0.01773959);
// khaki
pub const KHAKI_50: Srgba = Srgba::rgb(0.98305151, 0.97570442, 0.95283652);
pub const KHAKI_100: Srgba = Srgba::rgb(0.96545701, 0.94840679, 0.89568599);
pub const KHAKI_200: Srgba = Srgba::rgb(0.92276703, 0.89038807, 0.79117692);
pub const KHAKI_300: Srgba = Srgba::rgb(0.86551670, 0.81377002, 0.65644337);
pub const KHAKI_400: Srgba = Srgba::rgb(0.72896347, 0.66001791, 0.45008133);
pub const KHAKI_500: Srgba = Srgba::rgb(0.58736834, 0.51015724, 0.26932965);
pub const KHAKI_600: Srgba = Srgba::rgb(0.47136470, 0.39650404, 0.15316968);
pub const KHAKI_700: Srgba = Srgba::rgb(0.37813255, 0.31338030, 0.09819134);
pub const KHAKI_800: Srgba = Srgba::rgb(0.27720359, 0.22545395, 0.04912280);
pub const KHAKI_900: Srgba = Srgba::rgb(0.20956296, 0.16847841, 0.02959041);
pub const KHAKI_950: Srgba = Srgba::rgb(0.11151243, 0.08433130, 0.00590301);
// olive
pub const OLIVE_50: Srgba = Srgba::rgb(0.98017599, 0.97607751, 0.94733577);
pub const OLIVE_100: Srgba = Srgba::rgb(0.95885334, 0.94911859, 0.88280989);
pub const OLIVE_200: Srgba = Srgba::rgb(0.91269367, 0.89352080, 0.76845021);
pub const OLIVE_300: Srgba = Srgba::rgb(0.85029930, 0.81825100, 0.61889532);
pub const OLIVE_400: Srgba = Srgba::rgb(0.72094981, 0.67593692, 0.40569344);
pub const OLIVE_500: Srgba = Srgba::rgb(0.58848595, 0.53580995, 0.21524729);
pub const OLIVE_600: Srgba = Srgba::rgb(0.47595663, 0.42356402, 0.07756215);
pub const OLIVE_700: Srgba = Srgba::rgb(0.38274528, 0.33707419, 0.01509125);
pub const OLIVE_800: Srgba = Srgba::rgb(0.28613078, 0.24979365, 0.00000000);
pub const OLIVE_900: Srgba = Srgba::rgb(0.22107170, 0.19193883, 0.00225051);
pub const OLIVE_950: Srgba = Srgba::rgb(0.11895744, 0.10042737, 0.00000000);
// gold
pub const GOLD_50: Srgba = Srgba::rgb(0.99158444, 0.96885505, 0.92243932);
pub const GOLD_100: Srgba = Srgba::rgb(0.98435861, 0.93169449, 0.82418156);
pub const GOLD_200: Srgba = Srgba::rgb(0.96729117, 0.86731540, 0.66142495);
pub const GOLD_300: Srgba = Srgba::rgb(0.93322744, 0.77362575, 0.42894935);
pub const GOLD_400: Srgba = Srgba::rgb(0.86887571, 0.65545387, 0.00000000);
pub const GOLD_500: Srgba = Srgba::rgb(0.74482223, 0.56005761, 0.00000000);
pub const GOLD_600: Srgba = Srgba::rgb(0.61962780, 0.46378395, 0.00000000);
pub const GOLD_700: Srgba = Srgba::rgb(0.50422309, 0.37503853, 0.00000000);
pub const GOLD_800: Srgba = Srgba::rgb(0.40917485, 0.30194707, 0.00000000);
pub const GOLD_900: Srgba = Srgba::rgb(0.34100069, 0.24952159, 0.00000000);
pub const GOLD_950: Srgba = Srgba::rgb(0.19998912, 0.14108455, 0.00000000);
// bronze
pub const BRONZE_50: Srgba = Srgba::rgb(0.99420941, 0.96998931, 0.94550803);
pub const BRONZE_100: Srgba = Srgba::rgb(0.99045808, 0.93498510, 0.87856942);
pub const BRONZE_200: Srgba = Srgba::rgb(0.97100467, 0.86773957, 0.76147003);
pub const BRONZE_300: Srgba = Srgba::rgb(0.93795961, 0.77701983, 0.60774673);
pub const BRONZE_400: Srgba = Srgba::rgb(0.83511449, 0.62587249, 0.39538622);
pub const BRONZE_500: Srgba = Srgba::rgb(0.71324968, 0.48310319, 0.20553109);
pub const BRONZE_600: Srgba = Srgba::rgb(0.59388775, 0.37298275, 0.06190384);
pub const BRONZE_700: Srgba = Srgba::rgb(0.48377172, 0.29326791, 0.00067405);
pub const BRONZE_800: Srgba = Srgba::rgb(0.36794270, 0.21830038, 0.00000000);
pub const BRONZE_900: Srgba = Srgba::rgb(0.28987423, 0.16828787, 0.00131086);
pub const BRONZE_950: Srgba = Srgba::rgb(0.16307759, 0.08591856, 0.00000000);
// copper
pub const COPPER_50: Srgba = Srgba::rgb(1.00000000, 0.96438871, 0.94903926);
pub const COPPER_100: Srgba = Srgba::rgb(1.00000000, 0.92284444, 0.88940111);
pub const COPPER_200: Srgba = Srgba::rgb(1.00000000, 0.84532578, 0.77759103);
pub const COPPER_300: Srgba = Srgba::rgb(1.00000000, 0.73209547, 0.61281840);
pub const COPPER_400: Srgba = Srgba::rgb(0.94453690, 0.57416268, 0.40382504);
pub const COPPER_500: Srgba = Srgba::rgb(0.84539835, 0.43542458, 0.23467675);
pub const COPPER_600: Srgba = Srgba::rgb(0.72353498, 0.32736266, 0.11263302);
pub const COPPER_700: Srgba = Srgba::rgb(0.59576029, 0.25319004, 0.05485147);
pub const COPPER_800: Srgba = Srgba::rgb(0.47073139, 0.19518417, 0.03734466);
pub const COPPER_900: Srgba = Srgba::rgb(0.37937997, 0.15914272, 0.04041901);
pub const COPPER_950: Srgba = Srgba::rgb(0.22511014, 0.07928607, 0.00940618);
// clay
pub const CLAY_50: Srgba = Srgba::rgb(0.99792006, 0.96825782, 0.95809621);
pub const CLAY_100: Srgba = Srgba::rgb(0.99877149, 0.93116716, 0.90792653);
pub const CLAY_200: Srgba = Srgba::rgb(0.98368141, 0.85885400, 0.81574428);
pub const CLAY_300: Srgba = Srgba::rgb(0.95623616, 0.76362121, 0.69681330);
pub const CLAY_400: Srgba = Srgba::rgb(0.84532512, 0.59851245, 0.51258836);
pub const CLAY_500: Srgba = Srgba::rgb(0.71275142, 0.44488088, 0.35122194);
pub const CLAY_600: Srgba = Srgba::rgb(0.58892667, 0.33372229, 0.24397427);
pub const CLAY_700: Srgba = Srgba::rgb(0.47860133, 0.25898822, 0.18153166);
pub const CLAY_800: Srgba = Srgba::rgb(0.35950904, 0.18444003, 0.12253926);
pub const CLAY_900: Srgba = Srgba::rgb(0.27724005, 0.13798053, 0.08877991);
pub const CLAY_950: Srgba = Srgba::rgb(0.15595853, 0.06425114, 0.03212727);
// terracotta
pub const TERRACOTTA_50: Srgba = Srgba::rgb(1.00000000, 0.96470726, 0.95549535);
pub const TERRACOTTA_100: Srgba = Srgba::rgb(1.00000000, 0.92380330, 0.90389910);
pub const TERRACOTTA_200: Srgba = Srgba::rgb(1.00000000, 0.84490366, 0.80444178);
pub const TERRACOTTA_300: Srgba = Srgba::rgb(1.00000000, 0.73136464, 0.66196637);
pub const TERRACOTTA_400: Srgba = Srgba::rgb(0.92583295, 0.57076183, 0.48085122);
pub const TERRACOTTA_500: Srgba = Srgba::rgb(0.81688679, 0.42642675, 0.32955576);
pub const TERRACOTTA_600: Srgba = Srgba::rgb(0.69341389, 0.31761128, 0.22532871);
pub const TERRACOTTA_700: Srgba = Srgba::rgb(0.56930387, 0.24475124, 0.16518932);
pub const TERRACOTTA_800: Srgba = Srgba::rgb(0.44434225, 0.18375207, 0.11987728);
pub const TERRACOTTA_900: Srgba = Srgba::rgb(0.35440109, 0.14629691, 0.09523171);
pub const TERRACOTTA_950: Srgba = Srgba::rgb(0.20804062, 0.07040778, 0.03671534);
// brown
pub const BROWN_50: Srgba = Srgba::rgb(0.99071743, 0.97294978, 0.96166829);
pub const BROWN_100: Srgba = Srgba::rgb(0.98285883, 0.94219647, 0.91626747);
pub const BROWN_200: Srgba = Srgba::rgb(0.95237872, 0.87690213, 0.82842699);
pub const BROWN_300: Srgba = Srgba::rgb(0.91005165, 0.79290181, 0.71689536);
pub const BROWN_400: Srgba = Srgba::rgb(0.77274747, 0.62263002, 0.52371111);
pub const BROWN_500: Srgba = Srgba::rgb(0.62299079, 0.46088729, 0.35192320);
pub const BROWN_600: Srgba = Srgba::rgb(0.50046228, 0.34674378, 0.24156501);
pub const BROWN_700: Srgba = Srgba::rgb(0.40214612, 0.27008948, 0.17910229);
pub const BROWN_800: Srgba = Srgba::rgb(0.29028407, 0.18573854, 0.11306022);
pub const BROWN_900: Srgba = Srgba::rgb(0.21569134, 0.13296971, 0.07535080);
pub const BROWN_950: Srgba = Srgba::rgb(0.11471564, 0.06042111, 0.02430189);
// sepia
pub const SEPIA_50: Srgba = Srgba::rgb(0.98713806, 0.97508921, 0.96417576);
pub const SEPIA_100: Srgba = Srgba::rgb(0.97485051, 0.94721423, 0.92210702);
pub const SEPIA_200: Srgba = Srgba::rgb(0.93648330, 0.88502450, 0.83802890);
pub const SEPIA_300: Srgba = Srgba::rgb(0.88613443, 0.80595133, 0.73214958);
pub const SEPIA_400: Srgba = Srgba::rgb(0.73568397, 0.63271720, 0.53672822);
pub const SEPIA_500: Srgba = Srgba::rgb(0.57782890, 0.46663464, 0.36119652);
pub const SEPIA_600: Srgba = Srgba::rgb(0.45623535, 0.35084468, 0.24939120);
pub const SEPIA_700: Srgba = Srgba::rgb(0.36398245, 0.27346352, 0.18581939);
pub const SEPIA_800: Srgba = Srgba::rgb(0.25638336, 0.18493295, 0.11514383);
pub const SEPIA_900: Srgba = Srgba::rgb(0.18599648, 0.12961981, 0.07437901);
pub const SEPIA_950: Srgba = Srgba::rgb(0.09490507, 0.05794444, 0.02368723);
// taupe
pub const TAUPE_50: Srgba = Srgba::rgb(0.98336623, 0.97770691, 0.97341793);
pub const TAUPE_100: Srgba = Srgba::rgb(0.96641222, 0.95341781, 0.94355585);
pub const TAUPE_200: Srgba = Srgba::rgb(0.91781455, 0.89359786, 0.87517443);
pub const TAUPE_300: Srgba = Srgba::rgb(0.85757057, 0.81978902, 0.79094879);
pub const TAUPE_400: Srgba = Srgba::rgb(0.68267226, 0.63435004, 0.59727289);
pub const TAUPE_500: Srgba = Srgba::rgb(0.50743682, 0.45564394, 0.41566019);
pub const TAUPE_600: Srgba = Srgba::rgb(0.38548357, 0.33666676, 0.29880471);
pub const TAUPE_700: Srgba = Srgba::rgb(0.30259221, 0.26074059, 0.22822811);
pub const TAUPE_800: Srgba = Srgba::rgb(0.19969245, 0.16696395, 0.14145979);
pub const TAUPE_900: Srgba = Srgba::rgb(0.13504482, 0.10943331, 0.08943996);
pub const TAUPE_950: Srgba = Srgba::rgb(0.06072527, 0.04401477, 0.03143706);
// peach
pub const PEACH_50: Srgba = Srgba::rgb(0.99720412, 0.96876981, 0.95652617);
pub const PEACH_100: Srgba = Srgba::rgb(0.99719118, 0.93234424, 0.90428222);
pub const PEACH_200: Srgba = Srgba::rgb(0.98090961, 0.86105894, 0.80879993);
pub const PEACH_300: Srgba = Srgba::rgb(0.95223671, 0.76709324, 0.68558527);
pub const PEACH_400: Srgba = Srgba::rgb(0.84061063, 0.60309384, 0.49712312);
pub const PEACH_500: Srgba = Srgba::rgb(0.70801608, 0.45002573, 0.33282158);
pub const PEACH_600: Srgba = Srgba::rgb(0.58463636, 0.33876091, 0.22493627);
pub const PEACH_700: Srgba = Srgba::rgb(0.47496904, 0.26336836, 0.16460301);
pub const PEACH_800: Srgba = Srgba::rgb(0.35664680, 0.18795842, 0.10869424);
pub const PEACH_900: Srgba = Srgba::rgb(0.27495543, 0.14077283, 0.07785267);
pub const PEACH_950: Srgba = Srgba::rgb(0.15447899, 0.06611124, 0.02586499);
// coral
pub const CORAL_50: Srgba = Srgba::rgb(1.00000000, 0.96122814, 0.95292005);
pub const CORAL_100: Srgba = Srgba::rgb(1.00000000, 0.91544552, 0.89736748);
pub const CORAL_200: Srgba = Srgba::rgb(1.00000000, 0.83563333, 0.80078144);
pub const CORAL_300: Srgba = Srgba::rgb(1.00000000, 0.71569596, 0.65697046);
pub const CORAL_400: Srgba = Srgba::rgb(1.00000000, 0.54512968, 0.45802210);
pub const CORAL_500: Srgba = Srgba::rgb(0.94324344, 0.39205939, 0.29527650);
pub const CORAL_600: Srgba = Srgba::rgb(0.82177916, 0.28259183, 0.19200828);
pub const CORAL_700: Srgba = Srgba::rgb(0.68108707, 0.21255936, 0.13472667);
pub const CORAL_800: Srgba = Srgba::rgb(0.55091031, 0.17387546, 0.11079741);
pub const CORAL_900: Srgba = Srgba::rgb(0.45255023, 0.15182145, 0.10055950);
pub const CORAL_950: Srgba = Srgba::rgb(0.27458783, 0.07485553, 0.04111440);
// salmon
pub const SALMON_50: Srgba = Srgba::rgb(1.00000000, 0.96556757, 0.96026178);
pub const SALMON_100: Srgba = Srgba::rgb(1.00000000, 0.92599617, 0.91467370);
pub const SALMON_200: Srgba = Srgba::rgb(1.00000000, 0.84624286, 0.82320810);
pub const SALMON_300: Srgba = Srgba::rgb(0.98494500, 0.74069634, 0.70554168);
pub const SALMON_400: Srgba = Srgba::rgb(0.89082260, 0.57733346, 0.53527323);
pub const SALMON_500: Srgba = Srgba::rgb(0.76977527, 0.42773166, 0.38593540);
pub const SALMON_600: Srgba = Srgba::rgb(0.64547010, 0.31810765, 0.28123232);
pub const SALMON_700: Srgba = Srgba::rgb(0.52754272, 0.24536346, 0.21455107);
pub const SALMON_800: Srgba = Srgba::rgb(0.40459367, 0.17873365, 0.15440469);
pub const SALMON_900: Srgba = Srgba::rgb(0.31784596, 0.13774411, 0.11802776);
pub const SALMON_950: Srgba = Srgba::rgb(0.18327119, 0.06439728, 0.05178281);
// blush
pub const BLUSH_50: Srgba = Srgba::rgb(0.98971859, 0.97228191, 0.97664333);
pub const BLUSH_100: Srgba = Srgba::rgb(0.98063891, 0.94076956, 0.95085929);
pub const BLUSH_200: Srgba = Srgba::rgb(0.94687000, 0.87300652, 0.89206707);
pub const BLUSH_300: Srgba = Srgba::rgb(0.90153157, 0.78718355, 0.81748435);
pub const BLUSH_400: Srgba = Srgba::rgb(0.75414111, 0.60852862, 0.64857298);
pub const BLUSH_500: Srgba = Srgba::rgb(0.59639261, 0.44038319, 0.48499696);
pub const BLUSH_600: Srgba = Srgba::rgb(0.47299808, 0.32585306, 0.36908977);
pub const BLUSH_700: Srgba = Srgba::rgb(0.37815282, 0.25196224, 0.28937727);
pub const BLUSH_800: Srgba = Srgba::rgb(0.26732910, 0.16791831, 0.19775989);
pub const BLUSH_900: Srgba = Srgba::rgb(0.19457074, 0.11618259, 0.13981156);
pub const BLUSH_950: Srgba = Srgba::rgb(0.10041222, 0.04911082, 0.06475934);
// crimson
pub const CRIMSON_50: Srgba = Srgba::rgb(1.00000000, 0.95956448, 0.95918811);
pub const CRIMSON_100: Srgba = Srgba::rgb(1.00000000, 0.91163818, 0.91112062);
pub const CRIMSON_200: Srgba = Srgba::rgb(1.00000000, 0.82960682, 0.82984079);
pub const CRIMSON_300: Srgba = Srgba::rgb(1.00000000, 0.70513364, 0.70999616);
pub const CRIMSON_400: Srgba = Srgba::rgb(1.00000000, 0.53764276, 0.55991022);
pub const CRIMSON_500: Srgba = Srgba::rgb(0.98322907, 0.34621153, 0.41339047);
pub const CRIMSON_600: Srgba = Srgba::rgb(0.86101097, 0.23023941, 0.31804802);
pub const CRIMSON_700: Srgba = Srgba::rgb(0.71497904, 0.16351142, 0.24796607);
pub const CRIMSON_800: Srgba = Srgba::rgb(0.58190709, 0.14019736, 0.20275672);
pub const CRIMSON_900: Srgba = Srgba::rgb(0.48033968, 0.13026429, 0.17327706);
pub const CRIMSON_950: Srgba = Srgba::rgb(0.29327539, 0.06018517, 0.09047191);
// wine
pub const WINE_50: Srgba = Srgba::rgb(1.00000000, 0.96456681, 0.96724973);
pub const WINE_100: Srgba = Srgba::rgb(1.00000000, 0.92382184, 0.92987839);
pub const WINE_200: Srgba = Srgba::rgb(1.00000000, 0.84161829, 0.85561688);
pub const WINE_300: Srgba = Srgba::rgb(0.98247022, 0.73439648, 0.75945219);
pub const WINE_400: Srgba = Srgba::rgb(0.88738876, 0.56933633, 0.60715167);
pub const WINE_500: Srgba = Srgba::rgb(0.76581374, 0.41910169, 0.46714872);
pub const WINE_600: Srgba = Srgba::rgb(0.64157006, 0.30988918, 0.36079890);
pub const WINE_700: Srgba = Srgba::rgb(0.52415467, 0.23828462, 0.28367497);
pub const WINE_800: Srgba = Srgba::rgb(0.40187365, 0.17306839, 0.20991391);
pub const WINE_900: Srgba = Srgba::rgb(0.31568480, 0.13322584, 0.16211883);
pub const WINE_950: Srgba = Srgba::rgb(0.18183521, 0.06141558, 0.08110068);
// chartreuse
pub const CHARTREUSE_50: Srgba = Srgba::rgb(0.96495494, 0.97834646, 0.92398835);
pub const CHARTREUSE_100: Srgba = Srgba::rgb(0.92381173, 0.95345518, 0.82772981);
pub const CHARTREUSE_200: Srgba = Srgba::rgb(0.85704979, 0.90914163, 0.66924485);
pub const CHARTREUSE_300: Srgba = Srgba::rgb(0.76523146, 0.83962916, 0.44275830);
pub const CHARTREUSE_400: Srgba = Srgba::rgb(0.66103014, 0.74894670, 0.08383036);
pub const CHARTREUSE_500: Srgba = Srgba::rgb(0.56936405, 0.64734711, 0.00000000);
pub const CHARTREUSE_600: Srgba = Srgba::rgb(0.47392995, 0.53999329, 0.00000000);
pub const CHARTREUSE_700: Srgba = Srgba::rgb(0.38397663, 0.43880482, 0.00000000);
pub const CHARTREUSE_800: Srgba = Srgba::rgb(0.31190010, 0.35772594, 0.00000000);
pub const CHARTREUSE_900: Srgba = Srgba::rgb(0.25965373, 0.29895401, 0.00000000);
pub const CHARTREUSE_950: Srgba = Srgba::rgb(0.14805687, 0.17341876, 0.00000000);
// moss
pub const MOSS_50: Srgba = Srgba::rgb(0.96190253, 0.98173696, 0.95746636);
pub const MOSS_100: Srgba = Srgba::rgb(0.91654685, 0.96213768, 0.90637674);
pub const MOSS_200: Srgba = Srgba::rgb(0.83256814, 0.91791194, 0.81361512);
pub const MOSS_300: Srgba = Srgba::rgb(0.72250333, 0.85655489, 0.69290862);
pub const MOSS_400: Srgba = Srgba::rgb(0.55009257, 0.72604232, 0.51146188);
pub const MOSS_500: Srgba = Srgba::rgb(0.39542952, 0.59108514, 0.35242879);
pub const MOSS_600: Srgba = Srgba::rgb(0.28703886, 0.47666250, 0.24501859);
pub const MOSS_700: Srgba = Srgba::rgb(0.21872349, 0.38286822, 0.18216878);
pub const MOSS_800: Srgba = Srgba::rgb(0.15483638, 0.28635265, 0.12544678);
pub const MOSS_900: Srgba = Srgba::rgb(0.11652355, 0.22108596, 0.09320922);
pub const MOSS_950: Srgba = Srgba::rgb(0.05025199, 0.11954879, 0.03484335);
// sage
pub const SAGE_50: Srgba = Srgba::rgb(0.97265822, 0.98079567, 0.97249812);
pub const SAGE_100: Srgba = Srgba::rgb(0.94171872, 0.96043890, 0.94137342);
pub const SAGE_200: Srgba = Srgba::rgb(0.87325102, 0.90826639, 0.87268003);
pub const SAGE_300: Srgba = Srgba::rgb(0.78772962, 0.84263347, 0.78700366);
pub const SAGE_400: Srgba = Srgba::rgb(0.60112112, 0.67206876, 0.60052317);
pub const SAGE_500: Srgba = Srgba::rgb(0.42598673, 0.50303656, 0.42577855);
pub const SAGE_600: Srgba = Srgba::rgb(0.31031168, 0.38364592, 0.31043530);
pub const SAGE_700: Srgba = Srgba::rgb(0.23833638, 0.30141569, 0.23853924);
pub const SAGE_800: Srgba = Srgba::rgb(0.15321205, 0.20297793, 0.15349878);
pub const SAGE_900: Srgba = Srgba::rgb(0.10157518, 0.14076337, 0.10184750);
pub const SAGE_950: Srgba = Srgba::rgb(0.03917047, 0.06490886, 0.03941088);
// forest
pub const FOREST_50: Srgba = Srgba::rgb(0.95170738, 0.98362187, 0.96077005);
pub const FOREST_100: Srgba = Srgba::rgb(0.89244089, 0.96628177, 0.91397726);
pub const FOREST_200: Srgba = Srgba::rgb(0.78728866, 0.92735534, 0.83011589);
pub const FOREST_300: Srgba = Srgba::rgb(0.64555521, 0.87068845, 0.71953466);
pub const FOREST_400: Srgba = Srgba::rgb(0.44533411, 0.75446359, 0.55842117);
pub const FOREST_500: Srgba = Srgba::rgb(0.26174886, 0.63060330, 0.41535991);
pub const FOREST_600: Srgba = Srgba::rgb(0.12674125, 0.51707253, 0.31069091);
pub const FOREST_700: Srgba = Srgba::rgb(0.06217253, 0.41807502, 0.24014907);
pub const FOREST_800: Srgba = Srgba::rgb(0.02989730, 0.32016351, 0.17754289);
pub const FOREST_900: Srgba = Srgba::rgb(0.03047910, 0.25239784, 0.13851027);
pub const FOREST_950: Srgba = Srgba::rgb(0.00465194, 0.14075469, 0.06556750);
// jade
pub const JADE_50: Srgba = Srgba::rgb(0.93575587, 0.98584863, 0.96565566);
pub const JADE_100: Srgba = Srgba::rgb(0.85406714, 0.97095733, 0.92513664);
pub const JADE_200: Srgba = Srgba::rgb(0.71302910, 0.93931920, 0.85527416);
pub const JADE_300: Srgba = Srgba::rgb(0.50626260, 0.88779453, 0.75965920);
pub const JADE_400: Srgba = Srgba::rgb(0.17305503, 0.79681157, 0.63350884);
pub const JADE_500: Srgba = Srgba::rgb(0.00000000, 0.67597133, 0.52832856);
pub const JADE_600: Srgba = Srgba::rgb(0.00000000, 0.55602731, 0.43261101);
pub const JADE_700: Srgba = Srgba::rgb(0.00000000, 0.45023442, 0.34818632);
pub const JADE_800: Srgba = Srgba::rgb(0.00000000, 0.35828902, 0.27481218);
pub const JADE_900: Srgba = Srgba::rgb(0.00000000, 0.29367068, 0.22324554);
pub const JADE_950: Srgba = Srgba::rgb(0.00000000, 0.16844539, 0.12331336);
// mint
pub const MINT_50: Srgba = Srgba::rgb(0.95737640, 0.98299189, 0.97471747);
pub const MINT_100: Srgba = Srgba::rgb(0.90587241, 0.96518461, 0.94631940);
pub const MINT_200: Srgba = Srgba::rgb(0.80871893, 0.92116930, 0.88637886);
pub const MINT_300: Srgba = Srgba::rgb(0.68173184, 0.86201832, 0.80856160);
pub const MINT_400: Srgba = Srgba::rgb(0.47521853, 0.71952566, 0.65191613);
pub const MINT_500: Srgba = Srgba::rgb(0.28742960, 0.57270410, 0.50071871);
pub const MINT_600: Srgba = Srgba::rgb(0.16275384, 0.45547885, 0.38795045);
pub const MINT_700: Srgba = Srgba::rgb(0.10342469, 0.36403628, 0.30624458);
pub const MINT_800: Srgba = Srgba::rgb(0.04826843, 0.26414382, 0.21844298);
pub const MINT_900: Srgba = Srgba::rgb(0.02767418, 0.19795152, 0.16169786);
pub const MINT_950: Srgba = Srgba::rgb(0.00427299, 0.10357725, 0.07987433);
// seafoam
pub const SEAFOAM_50: Srgba = Srgba::rgb(0.95084574, 0.98345419, 0.98025276);
pub const SEAFOAM_100: Srgba = Srgba::rgb(0.89037419, 0.96609841, 0.95896173);
pub const SEAFOAM_200: Srgba = Srgba::rgb(0.77971560, 0.92423270, 0.91156570);
pub const SEAFOAM_300: Srgba = Srgba::rgb(0.63154397, 0.86628904, 0.84787006);
pub const SEAFOAM_400: Srgba = Srgba::rgb(0.40435066, 0.73296942, 0.71133726);
pub const SEAFOAM_500: Srgba = Srgba::rgb(0.17832834, 0.59384436, 0.57249697);
pub const SEAFOAM_600: Srgba = Srgba::rgb(0.00000000, 0.47542449, 0.45685086);
pub const SEAFOAM_700: Srgba = Srgba::rgb(0.00000000, 0.37922574, 0.36402065);
pub const SEAFOAM_800: Srgba = Srgba::rgb(0.00000000, 0.27944098, 0.26773001);
pub const SEAFOAM_900: Srgba = Srgba::rgb(0.00000000, 0.21374254, 0.20433209);
pub const SEAFOAM_950: Srgba = Srgba::rgb(0.00000000, 0.11356140, 0.10765894);
// aqua
pub const AQUA_50: Srgba = Srgba::rgb(0.92312603, 0.98516731, 0.99217740);
pub const AQUA_100: Srgba = Srgba::rgb(0.82321166, 0.96901363, 0.98572627);
pub const AQUA_200: Srgba = Srgba::rgb(0.64893078, 0.93682032, 0.96992582);
pub const AQUA_300: Srgba = Srgba::rgb(0.36066228, 0.88208796, 0.93768181);
pub const AQUA_400: Srgba = Srgba::rgb(0.00000000, 0.78283905, 0.84544816);
pub const AQUA_500: Srgba = Srgba::rgb(0.00000000, 0.67033814, 0.72454041);
pub const AQUA_600: Srgba = Srgba::rgb(0.00000000, 0.55680253, 0.60252064);
pub const AQUA_700: Srgba = Srgba::rgb(0.00000000, 0.45214497, 0.49004235);
pub const AQUA_800: Srgba = Srgba::rgb(0.00000000, 0.36594815, 0.39740432);
pub const AQUA_900: Srgba = Srgba::rgb(0.00000000, 0.30412274, 0.33095890);
pub const AQUA_950: Srgba = Srgba::rgb(0.00000000, 0.17624291, 0.19352305);
// mist
pub const MIST_50: Srgba = Srgba::rgb(0.97169001, 0.98028825, 0.98397128);
pub const MIST_100: Srgba = Srgba::rgb(0.93950813, 0.95931837, 0.96778929);
pub const MIST_200: Srgba = Srgba::rgb(0.86804400, 0.90518818, 0.92102195);
pub const MIST_300: Srgba = Srgba::rgb(0.77940137, 0.83785585, 0.86265842);
pub const MIST_400: Srgba = Srgba::rgb(0.58476811, 0.66063373, 0.69257497);
pub const MIST_500: Srgba = Srgba::rgb(0.40358439, 0.48640194, 0.52091462);
pub const MIST_600: Srgba = Srgba::rgb(0.28732336, 0.36650464, 0.39921983);
pub const MIST_700: Srgba = Srgba::rgb(0.21824012, 0.28646543, 0.31456539);
pub const MIST_800: Srgba = Srgba::rgb(0.13471945, 0.18863495, 0.21070962);
pub const MIST_900: Srgba = Srgba::rgb(0.08509255, 0.12755308, 0.14488099);
pub const MIST_950: Srgba = Srgba::rgb(0.02895943, 0.05600000, 0.06734269);
// steel
pub const STEEL_50: Srgba = Srgba::rgb(0.97053780, 0.97905024, 0.98825536);
pub const STEEL_100: Srgba = Srgba::rgb(0.93681597, 0.95639405, 0.97752717);
pub const STEEL_200: Srgba = Srgba::rgb(0.86444980, 0.90105890, 0.94045757);
pub const STEEL_300: Srgba = Srgba::rgb(0.77385787, 0.83123742, 0.89273722);
pub const STEEL_400: Srgba = Srgba::rgb(0.58514463, 0.65930409, 0.73831363);
pub const STEEL_500: Srgba = Srgba::rgb(0.41010116, 0.49069710, 0.57596987);
pub const STEEL_600: Srgba = Srgba::rgb(0.29555344, 0.37231940, 0.45310357);
pub const STEEL_700: Srgba = Srgba::rgb(0.22567520, 0.29172587, 0.36110058);
pub const STEEL_800: Srgba = Srgba::rgb(0.14415205, 0.19632340, 0.25094890);
pub const STEEL_900: Srgba = Srgba::rgb(0.09516482, 0.13628569, 0.17928068);
pub const STEEL_950: Srgba = Srgba::rgb(0.03515564, 0.06204932, 0.09021156);
// azure
pub const AZURE_50: Srgba = Srgba::rgb(0.94650412, 0.97467213, 1.00000000);
pub const AZURE_100: Srgba = Srgba::rgb(0.88316313, 0.94486892, 1.00000000);
pub const AZURE_200: Srgba = Srgba::rgb(0.77493465, 0.89449973, 1.00000000);
pub const AZURE_300: Srgba = Srgba::rgb(0.61094196, 0.82014101, 1.00000000);
pub const AZURE_400: Srgba = Srgba::rgb(0.38601700, 0.72652998, 1.00000000);
pub const AZURE_500: Srgba = Srgba::rgb(0.00000000, 0.62826853, 0.98318522);
pub const AZURE_600: Srgba = Srgba::rgb(0.00000000, 0.52383087, 0.82449856);
pub const AZURE_700: Srgba = Srgba::rgb(0.00000000, 0.42539107, 0.67492530);
pub const AZURE_800: Srgba = Srgba::rgb(0.00000000, 0.34651462, 0.55507736);
pub const AZURE_900: Srgba = Srgba::rgb(0.00000000, 0.28933917, 0.46820279);
pub const AZURE_950: Srgba = Srgba::rgb(0.00000000, 0.16721392, 0.28264113);
// periwinkle
pub const PERIWINKLE_50: Srgba = Srgba::rgb(0.97126949, 0.97201354, 1.00000000);
pub const PERIWINKLE_100: Srgba = Srgba::rgb(0.93890333, 0.94021769, 1.00000000);
pub const PERIWINKLE_200: Srgba = Srgba::rgb(0.87432314, 0.87574586, 0.99830045);
pub const PERIWINKLE_300: Srgba = Srgba::rgb(0.79077055, 0.79028444, 0.98112859);
pub const PERIWINKLE_400: Srgba = Srgba::rgb(0.64463407, 0.63900340, 0.88656175);
pub const PERIWINKLE_500: Srgba = Srgba::rgb(0.50629480, 0.49434147, 0.76593995);
pub const PERIWINKLE_600: Srgba = Srgba::rgb(0.39844904, 0.38305548, 0.64335085);
pub const PERIWINKLE_700: Srgba = Srgba::rgb(0.31633303, 0.30191303, 0.52628169);
pub const PERIWINKLE_800: Srgba = Srgba::rgb(0.23357920, 0.22152993, 0.40084388);
pub const PERIWINKLE_900: Srgba = Srgba::rgb(0.17886012, 0.16952785, 0.31232090);
pub const PERIWINKLE_950: Srgba = Srgba::rgb(0.09206037, 0.08542033, 0.17962267);
// lavender
pub const LAVENDER_50: Srgba = Srgba::rgb(0.97769665, 0.97474281, 0.99033635);
pub const LAVENDER_100: Srgba = Srgba::rgb(0.95329729, 0.94641430, 0.98218616);
pub const LAVENDER_200: Srgba = Srgba::rgb(0.89712725, 0.88396662, 0.95060037);
pub const LAVENDER_300: Srgba = Srgba::rgb(0.82563110, 0.80436803, 0.90826052);
pub const LAVENDER_400: Srgba = Srgba::rgb(0.66165136, 0.63291556, 0.76655573);
pub const LAVENDER_500: Srgba = Srgba::rgb(0.50142235, 0.46868625, 0.61333306);
pub const LAVENDER_600: Srgba = Srgba::rgb(0.38559757, 0.35341249, 0.49073438);
pub const LAVENDER_700: Srgba = Srgba::rgb(0.30375723, 0.27577891, 0.39378714);
pub const LAVENDER_800: Srgba = Srgba::rgb(0.21026865, 0.18781744, 0.28110274);
pub const LAVENDER_900: Srgba = Srgba::rgb(0.15049558, 0.13268294, 0.20636169);
pub const LAVENDER_950: Srgba = Srgba::rgb(0.07192683, 0.06006638, 0.10841470);
// lilac
pub const LILAC_50: Srgba = Srgba::rgb(0.98242601, 0.96934805, 0.99352051);
pub const LILAC_100: Srgba = Srgba::rgb(0.96391897, 0.93375887, 0.98920215);
pub const LILAC_200: Srgba = Srgba::rgb(0.92071925, 0.86398075, 0.96735783);
pub const LILAC_300: Srgba = Srgba::rgb(0.86195932, 0.77229691, 0.93371886);
pub const LILAC_400: Srgba = Srgba::rgb(0.72925822, 0.61102968, 0.82062071);
pub const LILAC_500: Srgba = Srgba::rgb(0.59172538, 0.46011559, 0.68996577);
pub const LILAC_600: Srgba = Srgba::rgb(0.47676788, 0.34940227, 0.56961166);
pub const LILAC_700: Srgba = Srgba::rgb(0.38297551, 0.27283195, 0.46263159);
pub const LILAC_800: Srgba = Srgba::rgb(0.28378645, 0.19568076, 0.34714286);
pub const LILAC_900: Srgba = Srgba::rgb(0.21688606, 0.14687315, 0.26731754);
pub const LILAC_950: Srgba = Srgba::rgb(0.11659963, 0.07026656, 0.14969771);
// plum
pub const PLUM_50: Srgba = Srgba::rgb(0.99364748, 0.96446933, 0.98565968);
pub const PLUM_100: Srgba = Srgba::rgb(0.98899909, 0.92238060, 0.97118329);
pub const PLUM_200: Srgba = Srgba::rgb(0.96779493, 0.84439333, 0.93605524);
pub const PLUM_300: Srgba = Srgba::rgb(0.93201474, 0.74085991, 0.88542784);
pub const PLUM_400: Srgba = Srgba::rgb(0.82605449, 0.57909707, 0.76992049);
pub const PLUM_500: Srgba = Srgba::rgb(0.70214664, 0.43161473, 0.64465521);
pub const PLUM_600: Srgba = Srgba::rgb(0.58262395, 0.32330544, 0.52997737);
pub const PLUM_700: Srgba = Srgba::rgb(0.47390295, 0.25030839, 0.42919671);
pub const PLUM_800: Srgba = Srgba::rgb(0.36183665, 0.18285169, 0.32627691);
pub const PLUM_900: Srgba = Srgba::rgb(0.28358328, 0.14087617, 0.25501531);
pub const PLUM_950: Srgba = Srgba::rgb(0.16086656, 0.06665627, 0.14227600);
// mauve
pub const MAUVE_50: Srgba = Srgba::rgb(0.98391908, 0.97494100, 0.98264959);
pub const MAUVE_100: Srgba = Srgba::rgb(0.96758485, 0.94698284, 0.96470871);
pub const MAUVE_200: Srgba = Srgba::rgb(0.92150896, 0.88313829, 0.91626729);
pub const MAUVE_300: Srgba = Srgba::rgb(0.86313290, 0.80332537, 0.85520640);
pub const MAUVE_400: Srgba = Srgba::rgb(0.69806493, 0.62146094, 0.68835696);
pub const MAUVE_500: Srgba = Srgba::rgb(0.53068287, 0.44832087, 0.52075663);
pub const MAUVE_600: Srgba = Srgba::rgb(0.40956277, 0.33175017, 0.40051956);
pub const MAUVE_700: Srgba = Srgba::rgb(0.32359416, 0.25683195, 0.31592981);
pub const MAUVE_800: Srgba = Srgba::rgb(0.22032933, 0.16787674, 0.21442676);
pub const MAUVE_900: Srgba = Srgba::rgb(0.15437428, 0.11314985, 0.14977738);
pub const MAUVE_950: Srgba = Srgba::rgb(0.07377695, 0.04681534, 0.07082431);
