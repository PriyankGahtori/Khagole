// Zi Wei Dou Shu charts from iztro (MIT, https://github.com/SylarLong/iztro), as regression
// fixtures for khagole-engine: tests/fixtures/ziwei_iztro.json.
//
//   mkdir /tmp/iztro && cd /tmp/iztro && npm install --ignore-scripts iztro@2.6.1
//   node path/to/scripts/make_ziwei_fixtures.cjs > path/to/tests/fixtures/ziwei_iztro.json
//
// Births in Beijing (UTC+8, civil time), mapped to the engine's codes. iztro's defaults: the
// year changes at the lunar New Year, a leap month splits at the 15th, the common school.
const path = require('node:path');
const iztro = (name) => require(path.join(process.cwd(), 'node_modules', name));
const { astro } = iztro('iztro');
const { kot } = iztro('iztro/lib/i18n');

const STARS = {
  ziweiMaj: 'zi_wei', tianjiMaj: 'tian_ji', taiyangMaj: 'tai_yang', wuquMaj: 'wu_qu',
  tiantongMaj: 'tian_tong', lianzhenMaj: 'lian_zhen', tianfuMaj: 'tian_fu', taiyinMaj: 'tai_yin',
  tanlangMaj: 'tan_lang', jumenMaj: 'ju_men', tianxiangMaj: 'tian_xiang', tianliangMaj: 'tian_liang',
  qishaMaj: 'qi_sha', pojunMaj: 'po_jun', zuofuMin: 'zuo_fu', youbiMin: 'you_bi',
  wenchangMin: 'wen_chang', wenquMin: 'wen_qu', tiankuiMin: 'tian_kui', tianyueMin: 'tian_yue',
  lucunMin: 'lu_cun', tianmaMin: 'tian_ma', qingyangMin: 'qing_yang', tuoluoMin: 'tuo_luo',
  huoxingMin: 'huo_xing', lingxingMin: 'ling_xing', dikongMin: 'di_kong', dijieMin: 'di_jie',
};
const MINOR = {
  hongluan: 'hong_luan', tianxi: 'tian_xi', tianyao: 'tian_yao', xianchi: 'xian_chi',
  jieshen: 'jie_shen', santai: 'san_tai', bazuo: 'ba_zuo', engguang: 'en_guang',
  tiangui: 'tian_gui', longchi: 'long_chi', fengge: 'feng_ge', tiancai: 'tian_cai',
  tianshou: 'tian_shou', taifu: 'tai_fu', fenggao: 'feng_gao', tianwu: 'tian_wu',
  huagai: 'hua_gai', tianguan: 'tian_guan', tianfu: 'tian_fu_blessing', tianchu: 'tian_chu',
  tianyue: 'tian_yue_moon', tiande: 'tian_de', yuede: 'yue_de', tiankong: 'tian_kong',
  xunkong: 'xun_kong', jielu: 'jie_lu', kongwang: 'kong_wang', guchen: 'gu_chen',
  guasu: 'gua_su', feilian: 'fei_lian', posui: 'po_sui', tianxing: 'tian_xing',
  yinsha: 'yin_sha', tianku: 'tian_ku', tianxu: 'tian_xu', tianshi: 'tian_shi',
  tianshang: 'tian_shang', nianjie: 'nian_jie',
};
const GODS = [
  { changsheng: 'birth', muyu: 'bath', guandai: 'cap_and_belt', linguan: 'coming_of_age', diwang: 'prosperity', shuai: 'decline', bing: 'sickness', si: 'death', mu: 'tomb', jue: 'extinction', tai: 'conception', yang: 'nurture' },
  { boshi: 'bo_shi', lishi: 'li_shi', qinglong: 'qing_long', xiaohao: 'xiao_hao', jiangjun: 'jiang_jun', zhoushu: 'zou_shu', faylian: 'fei_lian', xishen: 'xi_shen', bingfu: 'bing_fu', dahao: 'da_hao', fubing: 'fu_bing', guanfu: 'guan_fu' },
  { suijian: 'sui_jian', huiqi: 'hui_qi', sangmen: 'sang_men', guansuo: 'guan_suo', gwanfu: 'guan_fu', xiaohao: 'xiao_hao', dahao: 'da_hao', longde: 'long_de', baihu: 'bai_hu', tiande: 'tian_de', diaoke: 'diao_ke', bingfu: 'bing_fu' },
  { jiangxing: 'jiang_xing', panan: 'pan_an', suiyi: 'sui_yi', xiishen: 'xi_shen', huagai: 'hua_gai', jiesha: 'jie_sha', zhaisha: 'zai_sha', tiansha: 'tian_sha', zhibei: 'zhi_bei', xianchi: 'xian_chi', yuesha: 'yue_sha', wangshen: 'wang_shen' },
];
const MUTAGEN = { sihuaLu: 'lu', sihuaQuan: 'quan', sihuaKe: 'ke', sihuaJi: 'ji' };
const BRANCH = (name) => kot(name, 'Earthly').replace('Earthly', '').replace('woo', 'wu');
const STEM = (name) => kot(name, 'Heavenly').replace('Heavenly', '');

// A deterministic spread of births: every 23 days and 7 hours from 1950.
const cases = [];
let t = Date.UTC(1950, 0, 3, 0, 30);
while (cases.length < 120) {
  const local = new Date(t + 8 * 3600e3); // Beijing civil time
  const hour = local.getUTCHours();
  // Avoid the late Zi hour (23:00-24:00): iztro counts it with the next day's chart.
  if (hour !== 23) {
    const date = `${local.getUTCFullYear()}-${local.getUTCMonth() + 1}-${local.getUTCDate()}`;
    const timeIndex = Math.floor((hour + 1) / 2) % 12;
    const gender = cases.length % 2 ? 'female' : 'male';
    const chart = astro.bySolar(date, timeIndex, gender, true, 'zh-CN');
    cases.push({
      utc: new Date(t).toISOString().replace('.000Z', 'Z'),
      sex: gender,
      life_palace: BRANCH(chart.earthlyBranchOfSoulPalace),
      body_palace: BRANCH(chart.earthlyBranchOfBodyPalace),
      bureau: { 水二局: 2, 木三局: 3, 金四局: 4, 土五局: 5, 火六局: 6 }[chart.fiveElementsClass],
      palaces: chart.palaces.map((p) => ({
        branch: BRANCH(p.earthlyBranch),
        stem: STEM(p.heavenlyStem),
        stars: [...p.majorStars, ...p.minorStars]
          .map((s) => {
            const code = STARS[kot(s.name)];
            const brightness = s.brightness ? kot(s.brightness) : null;
            const mutagen = s.mutagen ? MUTAGEN[kot(s.mutagen)] : null;
            return `${code}:${brightness || ''}:${mutagen || ''}`;
          })
          .sort(),
        minor_stars: p.adjectiveStars.map((s) => MINOR[kot(s.name)]).sort(),
        gods: [p.changsheng12, p.boshi12, p.suiqian12, p.jiangqian12].map((g, i) => GODS[i][kot(g)]),
        decade: p.decadal.range,
        ages: p.ages,
      })),
    });
  }
  t += (23 * 24 + 7) * 3600e3;
}
process.stdout.write('[\n' + cases.map((c) => JSON.stringify(c)).join(',\n') + '\n]\n');
