const extensionName = "HTML Colour Utilities";
const version = "1.0.0";
const author = "@rscarson";
class Colour {
  constructor(values) {
    this.values = values;
  }
  /**
   * Convert this RGB object into an array
   * @returns integer
   */
  toArray() {
    return this.values;
  }
  /**
   * Convert an integer value into a colour
   * @param {int} value 
   * @returns Colour
   */
  static fromInt(value, nComponents) {
    let offset = (nComponents - 1) * 8;
    let output = new Colour([]);
    for (let i = 0; i < nComponents; i++) {
      output.values.push(value >> offset & 255);
      offset -= 8;
    }
    return output;
  }
  /**
   * Convert this colour into an integer
   * @returns integer
   */
  toInt() {
    let values = this.toArray();
    let output = 0;
    for (let v in values) {
      output <<= 8;
      output |= values[v];
    }
    return output;
  }
}
class RGBColour extends Colour {
  constructor(r, g, b) {
    super([r, g, b]);
  }
  r() {
    return this.values[0];
  }
  set_r(v) {
    this.values[0] = v;
  }
  g() {
    return this.values[1];
  }
  set_g(v) {
    this.values[1] = v;
  }
  b() {
    return this.values[2];
  }
  set_b(v) {
    this.values[2] = v;
  }
  /**
   * Convert an integer value into a colour
   * @param {int} value 
   * @returns Colour
   */
  static fromInt(value) {
    return new RGBColour(...super.fromInt(value, 3).values);
  }
  /**
   * Get an RGB value from an HSL colour
   * @param {HSLColour} value
   * @returns RGB object
   */
  static fromHSL(value) {
    function getColourComponent(v, t12, t22) {
      while (v > 1)
        v -= 1;
      while (v < 0)
        v += 1;
      if (6 * v < 1) {
        v = t22 + (t12 - t22) * 6 * v;
      } else if (2 * v < 1) {
        v = t12;
      } else if (3 * v < 2) {
        v = t22 + (t12 - t22) * (0.666 - v) * 6;
      } else {
        v = t22;
      }
      return Math.round(255 * v);
    }
    let [h, s, l] = [value.h(), value.s(), value.l()];
    if (s == 0) {
      let v = Math.round(l * 255);
      return new RGBColour(v, v, v);
    }
    let t1 = l < 0.5 ? l * (1 + s) : l + s - l * s;
    let t2 = 2 * l - t1;
    let pHue = h / 360;
    let r = getColourComponent(pHue + 0.333, t1, t2);
    let g = getColourComponent(pHue, t1, t2);
    let b = getColourComponent(pHue - 0.333, t1, t2);
    return new RGBColour(r, g, b);
  }
}
class HSLColour extends Colour {
  constructor(h, s, l) {
    super([h, s, l]);
  }
  h() {
    return this.values[0];
  }
  set_h(v) {
    this.values[0] = v;
  }
  s() {
    return this.values[1];
  }
  set_s(v) {
    this.values[1] = v;
  }
  l() {
    return this.values[2];
  }
  set_l(v) {
    this.values[2] = v;
  }
  /**
   * Convert an integer value into a colour
   * @param {int} value 
   * @returns Colour
   */
  static fromInt(value) {
    return new HSLColour(...super.fromInt(value, 3).values);
  }
  /**
   * Get an HSL value from an RGB colour
   * @param {RGBColour} value
   * @returns HSL object
   */
  static fromRGB(value) {
    let [pR, pG, pB] = value.toArray().map((e) => e / 255);
    let min = Math.min(pR, pG, pB);
    let max = Math.max(pR, pG, pB);
    let l = (min + max) / 2;
    let s = min == max ? 0 : l <= 0.5 ? (max - min) / (max + min) : (max + min) / (2 - max - min);
    let h = s == 0 ? 0 : max == pR ? (pG - pB) / (max - min) : max == pG ? 2 + (pB - pR) / (max - min) : 4 + (pR - pG) / (max - min);
    h *= 60;
    while (h < 0)
      h += 360;
    h = Math.round(h);
    s = Math.round(100 * s) / 100;
    l = Math.round(100 * l) / 100;
    return new HSLColour(h, s, l);
  }
}
const color_map = {
  "air_force_blue_raf": 6130344,
  "air_force_blue_usaf": 12431,
  "air_superiority_blue": 7512257,
  "alabama_crimson": 10692152,
  "alice_blue": 15792383,
  "alizarin_crimson": 14886454,
  "alloy_orange": 12870160,
  "almond": 15720141,
  "amaranth": 15018832,
  "amber": 16760576,
  "amber_sae_ece": 16743936,
  "american_rose": 16712510,
  "amethyst": 2412,
  "android_green": 10798649,
  "anti_flash_white": 15922164,
  "antique_brass": 13473141,
  "antique_fuchsia": 9526403,
  "antique_ruby": 8657709,
  "antique_white": 16444375,
  "ao_english": 32768,
  "apple_green": 9287168,
  "apricot": 16502449,
  "aqua": 255,
  "aquamarine": 8388564,
  "army_green": 4936480,
  "arsenic": 3884107,
  "arylide_yellow": 15324779,
  "ash_grey": 11714229,
  "asparagus": 8890731,
  "atomic_tangerine": 3990,
  "auburn": 10824234,
  "aureolin": 16641536,
  "aurometalsaurus": 7241600,
  "avocado": 5669379,
  "azure": 32767,
  "azure_mist_web": 15794175,
  "baby_blue": 9031664,
  "baby_blue_eyes": 10603249,
  "baby_pink": 16040642,
  "ball_blue": 2206669,
  "banana_mania": 16443317,
  "banana_yellow": 16769333,
  "barn_red": 8129026,
  "battleship_grey": 8684674,
  "bazaar": 9992059,
  "beau_blue": 12375270,
  "beaver": 10453360,
  "beige": 16119260,
  "big_dip_o_ruby": 10233154,
  "bisque": 16770244,
  "bistre": 4008735,
  "bittersweet": 16674654,
  "bittersweet_shimmer": 12537681,
  "black": 0,
  "black_bean": 4000770,
  "black_leather_jacket": 2438441,
  "black_olive": 3882038,
  "blanched_almond": 16772045,
  "blast_off_bronze": 10842468,
  "bleu_de_france": 3247335,
  "blizzard_blue": 11331054,
  "blond": 16445630,
  "blue": 15,
  "blue_bell": 10658512,
  "blue_crayola": 2061822,
  "blue_gray": 1692,
  "blue_green": 891066,
  "blue_munsell": 37807,
  "blue_ncs": 34749,
  "blue_pigment": 825,
  "blue_ryb": 149502,
  "blue_sapphire": 1204608,
  "blue_violet": 9055202,
  "blush": 14572931,
  "bole": 7947323,
  "bondi_blue": 38326,
  "bone": 14932681,
  "boston_university_red": 3072,
  "bottle_green": 27214,
  "boysenberry": 8860256,
  "brandeis_blue": 28927,
  "brass": 11904578,
  "brick_red": 13320532,
  "bright_cerulean": 1944790,
  "bright_green": 1776,
  "bright_lavender": 12555492,
  "bright_maroon": 12788040,
  "bright_pink": 16711807,
  "bright_turquoise": 583902,
  "bright_ube": 13737960,
  "brilliant_lavender": 16038911,
  "brilliant_rose": 16733603,
  "brink_pink": 16474239,
  "british_racing_green": 16933,
  "bronze": 13467442,
  "brown_traditional": 9849600,
  "brown_web": 10824234,
  "bubble_gum": 16761292,
  "bubbles": 15204095,
  "buff": 15785090,
  "bulgarian_rose": 4720135,
  "burgundy": 8388640,
  "burlywood": 14596231,
  "burnt_orange": 3152,
  "burnt_sienna": 15299665,
  "burnt_umber": 9057060,
  "byzantine": 12399524,
  "byzantium": 7350627,
  "cadet": 5466226,
  "cadet_blue": 6266528,
  "cadet_grey": 9544624,
  "cadmium_green": 27452,
  "cadmium_orange": 15566637,
  "cadmium_red": 14876706,
  "cadmium_yellow": 16774656,
  "caf_au_lait": 10910555,
  "caf_noir": 4929057,
  "cal_poly_green": 1985835,
  "cambridge_blue": 10731949,
  "camel": 12687979,
  "cameo_pink": 15711180,
  "camouflage_green": 7898731,
  "canary_yellow": 16772864,
  "candy_apple_red": 16713728,
  "candy_pink": 14971258,
  "capri": 49151,
  "caput_mortuum": 5842720,
  "cardinal": 12852794,
  "caribbean_green": 201,
  "carmine": 9830424,
  "carmine_m_p": 14090304,
  "carmine_pink": 15420482,
  "carmine_red": 16711736,
  "carnation_pink": 16754377,
  "carnelian": 11737883,
  "carolina_blue": 10074845,
  "carrot_orange": 15569185,
  "catalina_blue": 404088,
  "ceil": 9609679,
  "celadon": 11329967,
  "celadon_blue": 31655,
  "celadon_green": 3114108,
  "celeste_colour": 11730943,
  "celestial_blue": 4822992,
  "cerise": 14561635,
  "cerise_pink": 15481731,
  "cerulean": 31655,
  "cerulean_blue": 2773694,
  "cerulean_frost": 7183299,
  "cg_blue": 31397,
  "cg_red": 14695473,
  "chamoisee": 10516570,
  "champagne": 16438949,
  "charcoal": 3556687,
  "charm_pink": 15110060,
  "chartreuse_traditional": 14679808,
  "chartreuse_web": 8388352,
  "cherry": 14561635,
  "cherry_blossom_pink": 16758725,
  "chestnut": 13458524,
  "china_pink": 14577569,
  "china_rose": 11030894,
  "chinese_red": 11155486,
  "chocolate_traditional": 8077056,
  "chocolate_web": 13789470,
  "chrome_yellow": 16754432,
  "cinereous": 9994619,
  "cinnabar": 14893620,
  "cinnamon": 13789470,
  "citrine": 14995466,
  "classic_rose": 16501991,
  "cobalt": 18347,
  "cocoa_brown": 13789470,
  "coffee": 7294519,
  "columbia_blue": 10214911,
  "congo_pink": 16286585,
  "cool_black": 11875,
  "cool_grey": 9212588,
  "copper": 12088115,
  "copper_crayola": 14322279,
  "copper_penny": 11366249,
  "copper_red": 13331793,
  "copper_rose": 2406,
  "coquelicot": 16726016,
  "coral": 16744272,
  "coral_pink": 16286585,
  "coral_red": 16728128,
  "cordovan": 8994629,
  "corn": 16510045,
  "cornell_red": 11737883,
  "cornflower_blue": 6591981,
  "cornsilk": 16775388,
  "cosmic_latte": 16775399,
  "cotton_candy": 16760025,
  "cream": 16776656,
  "crimson": 14423100,
  "crimson_glory": 12451890,
  "cyan": 255,
  "cyan_process": 47083,
  "daffodil": 16777009,
  "dandelion": 15786288,
  "dark_blue": 139,
  "dark_brown": 6636321,
  "dark_byzantium": 6109524,
  "dark_candy_apple_red": 10747904,
  "dark_cerulean": 542078,
  "dark_chestnut": 9988448,
  "dark_coral": 13458245,
  "dark_cyan": 35723,
  "dark_electric_blue": 5466232,
  "dark_goldenrod": 12092939,
  "dark_gray": 11119017,
  "dark_green": 78368,
  "dark_imperial_blue": 16746,
  "dark_jungle_green": 1713185,
  "dark_khaki": 12433259,
  "dark_lava": 4734002,
  "dark_lavender": 7557014,
  "dark_magenta": 9109643,
  "dark_midnight_blue": 54,
  "dark_olive_green": 5597999,
  "dark_orange": 16747520,
  "dark_orchid": 10040012,
  "dark_pastel_blue": 7839435,
  "dark_pastel_green": 245820,
  "dark_pastel_purple": 9859030,
  "dark_pastel_red": 12729122,
  "dark_pink": 15160448,
  "dark_powder_blue": 57,
  "dark_raspberry": 8857175,
  "dark_red": 9109504,
  "dark_salmon": 15308410,
  "dark_scarlet": 5636889,
  "dark_sea_green": 9419919,
  "dark_sienna": 3937300,
  "dark_slate_blue": 4734347,
  "dark_slate_gray": 3100495,
  "dark_spring_green": 1536581,
  "dark_tan": 9535825,
  "dark_tangerine": 16754706,
  "dark_taupe": 4734002,
  "dark_terra_cotta": 13389404,
  "dark_turquoise": 52945,
  "dark_violet": 9699539,
  "dark_yellow": 10192652,
  "dartmouth_green": 28732,
  "davy_s_grey": 1365,
  "debian_red": 14092883,
  "deep_carmine": 11083838,
  "deep_carmine_pink": 15675448,
  "deep_carrot_orange": 15296812,
  "deep_cerise": 14299783,
  "deep_champagne": 16438949,
  "deep_chestnut": 12144200,
  "deep_coffee": 7356993,
  "deep_fuchsia": 12670145,
  "deep_jungle_green": 19273,
  "deep_lilac": 2395,
  "deep_magenta": 3084,
  "deep_peach": 16763812,
  "deep_pink": 16716947,
  "deep_ruby": 8666971,
  "deep_saffron": 3987,
  "deep_sky_blue": 49151,
  "deep_tuscan_red": 6701645,
  "denim": 1401021,
  "desert": 12687979,
  "desert_sand": 15583663,
  "dim_gray": 6908265,
  "dodger_blue": 2003199,
  "dogwood_rose": 14096488,
  "dollar_bill": 8764261,
  "drab": 9859351,
  "duke_blue": 156,
  "earth_yellow": 14788959,
  "ebony": 5594448,
  "ecru": 12759680,
  "eggplant": 6373457,
  "eggshell": 15788758,
  "egyptian_blue": 1062054,
  "electric_blue": 8255999,
  "electric_crimson": 16711743,
  "electric_cyan": 255,
  "electric_green": 240,
  "electric_indigo": 7274751,
  "electric_lavender": 16038911,
  "electric_lime": 3312,
  "electric_purple": 12517631,
  "electric_ultramarine": 4129023,
  "electric_violet": 9371903,
  "electric_yellow": 4080,
  "emerald": 5294200,
  "english_lavender": 11830165,
  "eton_blue": 9881762,
  "fallow": 12687979,
  "falu_red": 8394776,
  "fandango": 11875209,
  "fashion_fuchsia": 15990945,
  "fawn": 15051376,
  "feldgrau": 5070163,
  "fern_green": 5208386,
  "ferrari_red": 16721920,
  "field_drab": 7099422,
  "fire_engine_red": 13508649,
  "firebrick": 11674146,
  "flame": 14833698,
  "flamingo_pink": 16551596,
  "flavescent": 16247182,
  "flax": 15654018,
  "floral_white": 16775920,
  "fluorescent_orange": 16760576,
  "fluorescent_pink": 16716947,
  "fluorescent_yellow": 3312,
  "folly": 16711759,
  "forest_green_traditional": 82977,
  "forest_green_web": 2263842,
  "french_beige": 10910555,
  "french_blue": 29371,
  "french_lilac": 8806542,
  "french_lime": 3312,
  "french_raspberry": 13053e3,
  "french_rose": 16140938,
  "fuchsia": 3855,
  "fuchsia_crayola": 12670145,
  "fuchsia_pink": 3967,
  "fuchsia_rose": 13058933,
  "fulvous": 14976e3,
  "fuzzy_wuzzy": 3174,
  "gainsboro": 14474460,
  "gamboge": 14981903,
  "ghost_white": 16316671,
  "ginger": 11560192,
  "glaucous": 6324918,
  "glitter": 15132922,
  "gold_metallic": 13938487,
  "gold_web_golden": 16766720,
  "golden_brown": 10052885,
  "golden_poppy": 16564736,
  "golden_yellow": 16768768,
  "goldenrod": 14329120,
  "granny_smith_apple": 11068576,
  "gray": 8421504,
  "gray_asparagus": 4610373,
  "gray_html_css_gray": 8421504,
  "gray_x11_gray": 12500670,
  "green_color_wheel_x11_green": 240,
  "green_crayola": 1879160,
  "green_html_css_green": 32768,
  "green_munsell": 43127,
  "green_ncs": 40811,
  "green_pigment": 42320,
  "green_ryb": 6729778,
  "green_yellow": 11403055,
  "grullo": 11115142,
  "guppie_green": 65407,
  "halay_be": 6699092,
  "han_blue": 4484303,
  "han_purple": 5380346,
  "hansa_yellow": 15324779,
  "harlequin": 4194048,
  "harvard_crimson": 13172758,
  "harvest_gold": 14323968,
  "heart_gold": 8421376,
  "heliotrope": 14644223,
  "hollywood_cerise": 15990945,
  "honeydew": 15794160,
  "honolulu_blue": 32703,
  "hooker_s_green": 4815211,
  "hot_magenta": 16719310,
  "hot_pink": 16738740,
  "hunter_green": 3497531,
  "iceberg": 7448274,
  "icterine": 16578398,
  "imperial_blue": 9109,
  "inchworm": 11725917,
  "india_green": 1280008,
  "indian_red": 13458524,
  "indian_yellow": 14919767,
  "indigo": 7274751,
  "indigo_dye": 16746,
  "indigo_web": 4915330,
  "international_klein_blue": 12199,
  "international_orange_aerospace": 16731904,
  "international_orange_engineering": 12195340,
  "international_orange_golden_gate_bridge": 12596780,
  "iris": 5918671,
  "isabelline": 16052460,
  "islamic_green": 36864,
  "ivory": 16777200,
  "jade": 43115,
  "jasmine": 16309886,
  "jasper": 14105406,
  "jazzberry_jam": 10816350,
  "jet": 3421236,
  "jonquil": 16439902,
  "june_bud": 12442199,
  "jungle_green": 2730887,
  "kelly_green": 5028631,
  "kenyan_copper": 8133637,
  "khaki_html_css_khaki": 12824721,
  "khaki_x11_light_khaki": 15787660,
  "ku_crimson": 15204365,
  "la_salle_green": 555056,
  "languid_lavender": 14076637,
  "lapis_lazuli": 2515356,
  "laser_lemon": 16711202,
  "laurel_green": 11123357,
  "lava": 13570080,
  "lavender_blue": 3279,
  "lavender_blush": 16773365,
  "lavender_floral": 11894492,
  "lavender_gray": 12895184,
  "lavender_indigo": 9721835,
  "lavender_magenta": 15631086,
  "lavender_mist": 15132410,
  "lavender_pink": 16494290,
  "lavender_purple": 9862070,
  "lavender_rose": 16490723,
  "lavender_web": 15132410,
  "lawn_green": 8190976,
  "lemon": 16774912,
  "lemon_chiffon": 16775885,
  "lemon_lime": 14941952,
  "licorice": 1708304,
  "light_apricot": 16635313,
  "light_blue": 11393254,
  "light_brown": 11887901,
  "light_carmine_pink": 15099761,
  "light_coral": 15761536,
  "light_cornflower_blue": 9686250,
  "light_crimson": 16083345,
  "light_cyan": 14745599,
  "light_fuchsia_pink": 16352495,
  "light_goldenrod_yellow": 16448210,
  "light_gray": 13882323,
  "light_green": 9498256,
  "light_khaki": 15787660,
  "light_pastel_purple": 11640025,
  "light_pink": 16758465,
  "light_red_ochre": 15299665,
  "light_salmon": 16752762,
  "light_salmon_pink": 3993,
  "light_sea_green": 2142890,
  "light_sky_blue": 8900346,
  "light_slate_gray": 1929,
  "light_taupe": 11766637,
  "light_thulian_pink": 15110060,
  "light_yellow": 16777184,
  "lilac": 13148872,
  "lime_color_wheel": 12582656,
  "lime_green": 3329330,
  "lime_web_x11_green": 240,
  "limerick": 10338825,
  "lincoln_green": 1661189,
  "linen": 16445670,
  "lion": 12687979,
  "little_boy_blue": 7119068,
  "liver": 5458767,
  "lust": 15081504,
  "magenta": 3855,
  "magenta_dye": 13246331,
  "magenta_process": 16711824,
  "magic_mint": 11202769,
  "magnolia": 16315647,
  "mahogany": 12599296,
  "maize": 16510045,
  "majorelle_blue": 6312156,
  "malachite": 776785,
  "manatee": 9935530,
  "mango_tango": 16745027,
  "mantis": 7652197,
  "mardi_gras": 8913029,
  "maroon_crayola": 12788040,
  "maroon_html_css": 8388608,
  "maroon_x11": 11546720,
  "mauve": 14725375,
  "mauve_taupe": 9527149,
  "mauvelous": 15702186,
  "maya_blue": 7586555,
  "meat_brown": 15054651,
  "medium_aquamarine": 1754,
  "medium_blue": 205,
  "medium_candy_apple_red": 14812716,
  "medium_carmine": 11485237,
  "medium_champagne": 15984043,
  "medium_electric_blue": 217238,
  "medium_jungle_green": 1848621,
  "medium_lavender_magenta": 14524637,
  "medium_orchid": 12211667,
  "medium_persian_blue": 26533,
  "medium_purple": 9662683,
  "medium_red_violet": 12268421,
  "medium_ruby": 11157609,
  "medium_sea_green": 3978097,
  "medium_slate_blue": 8087790,
  "medium_spring_bud": 13229191,
  "medium_spring_green": 64154,
  "medium_taupe": 6769735,
  "medium_turquoise": 4772300,
  "medium_tuscan_red": 7947323,
  "medium_vermilion": 14245947,
  "medium_violet_red": 13047173,
  "mellow_apricot": 16300152,
  "mellow_yellow": 16309886,
  "melon": 16628916,
  "midnight_blue": 1644912,
  "midnight_green_eagle_green": 18771,
  "mikado_yellow": 16761868,
  "mint": 4109449,
  "mint_cream": 16121850,
  "mint_green": 10026904,
  "misty_rose": 16770273,
  "moccasin": 16444375,
  "mode_beige": 9859351,
  "moonstone_blue": 7580098,
  "mordant_red_19": 11406336,
  "moss_green": 11394989,
  "mountain_meadow": 3193487,
  "mountbatten_pink": 10058381,
  "msu_green": 1590587,
  "mulberry": 12929932,
  "mustard": 16767832,
  "myrtle": 2179614,
  "nadeshiko_pink": 16166342,
  "napier_green": 2785280,
  "naples_yellow": 16439902,
  "navajo_white": 16768685,
  "navy_blue": 128,
  "neon_carrot": 16753475,
  "neon_fuchsia": 16662884,
  "neon_green": 3800852,
  "new_york_pink": 14123903,
  "non_photo_blue": 10804717,
  "north_texas_green": 364595,
  "ocean_boat_blue": 30654,
  "ochre": 3186,
  "office_green": 32768,
  "old_gold": 13612347,
  "old_lace": 16643558,
  "old_lavender": 7956600,
  "old_mauve": 6762823,
  "old_rose": 12615809,
  "olive": 8421376,
  "olive_drab_7": 3945503,
  "olive_drab_web_olive_drab_3": 7048739,
  "olivine": 10140019,
  "onyx": 3487801,
  "opera_mauve": 12027047,
  "orange_color_wheel": 16744192,
  "orange_peel": 16752384,
  "orange_red": 16729344,
  "orange_ryb": 16488706,
  "orange_web_color": 16753920,
  "orchid": 14315734,
  "otter_brown": 6636321,
  "ou_crimson_red": 2304,
  "outer_space": 4278860,
  "outrageous_orange": 16739914,
  "oxford_blue": 8519,
  "pakistan_green": 96,
  "palatinate_blue": 2571234,
  "palatinate_purple": 6826080,
  "pale_aqua": 12375270,
  "pale_blue": 11529966,
  "pale_brown": 9991764,
  "pale_carmine": 11485237,
  "pale_cerulean": 10208482,
  "pale_chestnut": 14527919,
  "pale_copper": 14322279,
  "pale_cornflower_blue": 11259375,
  "pale_gold": 15122058,
  "pale_goldenrod": 15657130,
  "pale_green": 10025880,
  "pale_lavender": 14471423,
  "pale_magenta": 16352485,
  "pale_pink": 16440029,
  "pale_plum": 14524637,
  "pale_red_violet": 14381203,
  "pale_robin_egg_blue": 9887441,
  "pale_silver": 13222075,
  "pale_spring_bud": 15526845,
  "pale_taupe": 12359806,
  "pale_violet_red": 14381203,
  "pansy_purple": 7870538,
  "papaya_whip": 16773077,
  "paris_green": 5294200,
  "pastel_blue": 11454159,
  "pastel_brown": 8612179,
  "pastel_gray": 13619140,
  "pastel_green": 2007,
  "pastel_magenta": 16030402,
  "pastel_orange": 16757575,
  "pastel_pink": 14591396,
  "pastel_purple": 11771573,
  "pastel_red": 16738657,
  "pastel_violet": 13343177,
  "pastel_yellow": 16645526,
  "patriarch": 8388736,
  "payne_s_grey": 5466232,
  "peach": 16770484,
  "peach_crayola": 16763812,
  "peach_orange": 4041,
  "peach_puff": 16767673,
  "peach_yellow": 16441261,
  "pear": 13754929,
  "pearl": 15392968,
  "pearl_aqua": 8968384,
  "pearly_purple": 12019874,
  "peridot": 15131136,
  "periwinkle": 3279,
  "persian_blue": 1849787,
  "persian_green": 42643,
  "persian_indigo": 3281530,
  "persian_orange": 14258264,
  "persian_pink": 16220094,
  "persian_plum": 7347228,
  "persian_red": 3123,
  "persian_rose": 16656546,
  "persimmon": 15489024,
  "peru": 13468991,
  "phlox": 14614783,
  "phthalo_blue": 3977,
  "phthalo_green": 1193252,
  "piggy_pink": 16637414,
  "pine_green": 96623,
  "pink": 16761035,
  "pink_lace": 16768500,
  "pink_orange": 3990,
  "pink_pearl": 15183055,
  "pink_sherbet": 16224167,
  "pistachio": 9684338,
  "platinum": 15066338,
  "plum_traditional": 9323909,
  "plum_web": 14524637,
  "portland_orange": 16734774,
  "powder_blue_web": 11591910,
  "princeton_orange": 16748288,
  "prune": 7347228,
  "prussian_blue": 12627,
  "psychedelic_purple": 14614783,
  "puce": 3209,
  "pumpkin": 16741656,
  "purple_heart": 6895004,
  "purple_html_css": 8388736,
  "purple_mountain_majesty": 9861302,
  "purple_munsell": 10420421,
  "purple_pizzazz": 16666330,
  "purple_taupe": 5259341,
  "purple_x11": 10494192,
  "quartz": 5326927,
  "rackley": 6130344,
  "radical_red": 16725342,
  "rajah": 16493408,
  "raspberry": 14879581,
  "raspberry_glace": 9527149,
  "raspberry_pink": 14831768,
  "raspberry_rose": 11748460,
  "raw_umber": 8545860,
  "razzle_dazzle_rose": 3900,
  "razzmatazz": 14886251,
  "red": 3840,
  "red_brown": 10824234,
  "red_devil": 8782097,
  "red_munsell": 15859772,
  "red_ncs": 12845619,
  "red_orange": 16733001,
  "red_pigment": 15539236,
  "red_ryb": 16656146,
  "red_violet": 13047173,
  "redwood": 11226706,
  "regalia": 5385600,
  "resolution_blue": 9095,
  "rich_black": 16448,
  "rich_brilliant_lavender": 15837182,
  "rich_carmine": 14090304,
  "rich_electric_blue": 561872,
  "rich_lavender": 10972111,
  "rich_lilac": 11953874,
  "rich_maroon": 11546720,
  "rifle_green": 4278323,
  "robin_egg_blue": 204,
  "rose": 16711807,
  "rose_bonbon": 16335518,
  "rose_ebony": 6768710,
  "rose_gold": 12021369,
  "rose_madder": 14886454,
  "rose_pink": 3948,
  "rose_quartz": 11180201,
  "rose_taupe": 9461085,
  "rose_vale": 11226706,
  "rosewood": 6619147,
  "rosso_corsa": 13893632,
  "rosy_brown": 12357519,
  "royal_azure": 14504,
  "royal_blue_traditional": 9062,
  "royal_blue_web": 4286945,
  "royal_fuchsia": 13249682,
  "royal_purple": 7885225,
  "royal_yellow": 16439902,
  "rubine_red": 13697110,
  "ruby": 14684511,
  "ruby_red": 10162462,
  "ruddy": 16711720,
  "ruddy_brown": 12281128,
  "ruddy_pink": 14782102,
  "rufous": 11017223,
  "russet": 8406555,
  "rust": 12009742,
  "rusty_red": 14298179,
  "sacramento_state_green": 22079,
  "saddle_brown": 9127187,
  "safety_orange_blaze_orange": 16738048,
  "saffron": 16041008,
  "salmon": 16747625,
  "salmon_pink": 16748964,
  "sand": 12759680,
  "sand_dune": 9859351,
  "sandstorm": 15521088,
  "sandy_brown": 16032864,
  "sandy_taupe": 9859351,
  "sangria": 9568266,
  "sap_green": 5274922,
  "sapphire": 1004218,
  "sapphire_blue": 26533,
  "satin_sheen_gold": 13345077,
  "scarlet": 16720896,
  "scarlet_crayola": 16584245,
  "school_bus_yellow": 16766976,
  "screamin_green": 7798650,
  "sea_blue": 27028,
  "sea_green": 3050327,
  "seal_brown": 3281940,
  "seashell": 16774638,
  "selective_yellow": 16759296,
  "sepia": 7356948,
  "shadow": 9075037,
  "shamrock_green": 40544,
  "shocking_pink": 16519104,
  "shocking_pink_crayola": 16740351,
  "sienna": 8924439,
  "silver": 12632256,
  "sinopia": 13320459,
  "skobeloff": 29812,
  "sky_blue": 8900331,
  "sky_magenta": 13595055,
  "slate_blue": 6970061,
  "slate_gray": 7372944,
  "smalt_dark_powder_blue": 57,
  "smokey_topaz": 9649473,
  "smoky_black": 1051656,
  "snow": 16775930,
  "spiro_disco_ball": 1032444,
  "spring_bud": 11009024,
  "spring_green": 65407,
  "st_patrick_s_blue": 2304378,
  "steel_blue": 4620980,
  "stil_de_grain_yellow": 16439902,
  "stizza": 2304,
  "stormcloud": 5203562,
  "straw": 14997871,
  "sunglow": 4035,
  "sunset": 16438949,
  "tan": 13808780,
  "tangelo": 16338176,
  "tangerine": 15893760,
  "tangerine_yellow": 4032,
  "tango_pink": 14971258,
  "taupe": 4734002,
  "taupe_gray": 9143689,
  "tea_green": 13693120,
  "tea_rose_orange": 16286585,
  "tea_rose_rose": 16040642,
  "teal": 32896,
  "teal_blue": 3569032,
  "teal_green": 33407,
  "telemagenta": 13579382,
  "tenn_tawny": 13457152,
  "terra_cotta": 14840411,
  "thistle": 14204888,
  "thulian_pink": 14577569,
  "tickle_me_pink": 16550316,
  "tiffany_blue": 703157,
  "tiger_s_eye": 14716220,
  "timberwolf": 14407634,
  "titanium_yellow": 15656448,
  "tomato": 16737095,
  "toolbox": 7630016,
  "topaz": 16763004,
  "tractor_red": 16584245,
  "trolley_grey": 8421504,
  "tropical_rain_forest": 30046,
  "true_blue": 29647,
  "tufts_blue": 4292033,
  "tumbleweed": 14592648,
  "turkish_rose": 11891329,
  "turquoise": 3200456,
  "turquoise_blue": 65519,
  "turquoise_green": 10540724,
  "tuscan_red": 8144968,
  "twilight_lavender": 9062763,
  "tyrian_purple": 6685244,
  "ua_blue": 58,
  "ua_red": 14221388,
  "ube": 8943811,
  "ucla_blue": 5466261,
  "ucla_gold": 16757504,
  "ufo_green": 3985520,
  "ultra_pink": 16740351,
  "ultramarine": 1182351,
  "ultramarine_blue": 4286197,
  "umber": 6508871,
  "unbleached_silk": 16768458,
  "united_nations_blue": 6001381,
  "university_of_california_gold": 12027687,
  "unmellow_yellow": 4086,
  "up_forest_green": 82977,
  "up_maroon": 8065299,
  "upsdell_red": 11411497,
  "urobilin": 14789921,
  "usafa_blue": 20376,
  "usc_cardinal": 2304,
  "usc_gold": 4032,
  "utah_crimson": 13828159,
  "vanilla": 15984043,
  "vegas_gold": 12956504,
  "venetian_red": 13109269,
  "verdigris": 4436910,
  "vermilion_cinnabar": 14893620,
  "vermilion_plochere": 14245947,
  "veronica": 10494192,
  "violet": 9371903,
  "violet_blue": 3295922,
  "violet_color_wheel": 8323327,
  "violet_ryb": 8782255,
  "violet_web": 15631086,
  "viridian": 4227693,
  "vivid_auburn": 9578276,
  "vivid_burgundy": 10427701,
  "vivid_cerise": 14294401,
  "vivid_tangerine": 16752777,
  "vivid_violet": 10420479,
  "warm_black": 16962,
  "waterspout": 10810617,
  "wenge": 6575186,
  "wheat": 16113331,
  "white": 4095,
  "white_smoke": 16119285,
  "wild_blue_yonder": 10661328,
  "wild_strawberry": 16728996,
  "wild_watermelon": 16542853,
  "wine": 7483191,
  "wine_dregs": 6762823,
  "wisteria": 13213916,
  "wood_brown": 12687979,
  "xanadu": 7571064,
  "yale_blue": 1002898,
  "yellow": 4080,
  "yellow_green": 10145074,
  "yellow_munsell": 15715328,
  "yellow_ncs": 16765696,
  "yellow_orange": 16756290,
  "yellow_process": 16772864,
  "yellow_ryb": 16711219,
  "zaffre": 5288,
  "zinnwaldite_brown": 2889224
};
globalThis.extension = () => {
  return {
    name: `${extensionName}`,
    author: `${author}`,
    version: `${version}`,
    functions: {
      "complement": "functionComplement",
      "colour": "functionColour",
      "color": "functionColour",
      "int_to_rgb": "functionIntToRGB",
      "rgb_to_int": "functionRGBToInt",
      "rgb_to_hsl": "functionRGBToHSL",
      "hsl_to_rgb": "functionHSLToRGB",
      "hsl_to_int": "functionHSLToInt"
    },
    decorators: {
      "color": "decoratorColour",
      "colour": "decoratorColour"
    }
  };
};
lavendeuxExtensionName(`${extensionName}`);
lavendeuxExtensionAuthor(`${author}`);
lavendeuxExtensionVersion(`${version}`);
lavendeuxDecorator(
  "color",
  (input) => `#${(input & 16777215).toString(16).padEnd(6, "0")}`,
  lavendeuxType.Int
);
lavendeuxFunction("color", (name) => {
  if (color_map[name]) {
    return color_map[name];
  }
  return 0;
}, {
  description: "Converts a colour name to a hex code.",
  arguments: [lavendeuxType.String],
  returns: lavendeuxType.Int
});
lavendeuxFunction("complement", (input) => {
  let rgb = RGBColour.fromInt(input);
  rgb.set_r(255 - rgb.r());
  rgb.set_g(255 - rgb.g());
  rgb.set_b(255 - rgb.b());
  rgb.toInt();
}, {
  description: "Returns the complement of a colour.",
  arguments: [lavendeuxType.Int],
  returns: lavendeuxType.Int
});
lavendeuxFunction("int_to_rgb", (input) => {
  let rgb = RGBColour.fromInt(input);
  return [rgb.r(), rgb.g(), rgb.b()];
}, {
  description: "Converts a colour integer to an RGB array.",
  arguments: [lavendeuxType.Int],
  returns: lavendeuxType.Array
});
lavendeuxFunction("rgb_to_int", (input) => {
  let rgb = new RGBColour(...input);
  return rgb.toInt();
}, {
  description: "Converts an RGB array to a colour integer.",
  arguments: [lavendeuxType.Array],
  returns: lavendeuxType.Int
});
lavendeuxFunction("rgb_to_hsl", (input) => {
  let rgb = new RGBColour(...input);
  let hsl = rgb.toHSL();
  return [hsl.h(), hsl.s(), hsl.l()];
}, {
  description: "Converts an RGB array to an HSL array.",
  arguments: [lavendeuxType.Array],
  returns: lavendeuxType.Array
});
lavendeuxFunction("hsl_to_rgb", (input) => {
  let hsl = new HSLColour(...input);
  let rgb = hsl.toRGB();
  return [rgb.r(), rgb.g(), rgb.b()];
}, {
  description: "Converts an HSL array to an RGB array.",
  arguments: [lavendeuxType.Array],
  returns: lavendeuxType.Array
});
lavendeuxFunction("hsl_to_int", (input) => {
  let hsl = new HSLColour(...input);
  let rgb = hsl.toRGB();
  return rgb.toInt();
}, {
  description: "Converts an HSL array to a colour integer.",
  arguments: [lavendeuxType.Array],
  returns: lavendeuxType.Int
});
