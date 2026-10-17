// Zi Wei Dou Shu horoscopes from iztro (MIT, https://github.com/SylarLong/iztro), as
// regression fixtures for khagole-engine: tests/fixtures/ziwei_iztro_horoscope.json.
//
//   mkdir /tmp/iztro && cd /tmp/iztro && npm install --ignore-scripts iztro@2.6.1
//   node path/to/scripts/make_ziwei_horoscope_fixtures.cjs > path/to/tests/fixtures/ziwei_iztro_horoscope.json
//
// Births in Beijing (UTC+8, civil time); target dates at noon, after the birth. Palaces are
// given by branch.
const path = require('node:path');
const iztro = (name) => require(path.join(process.cwd(), 'node_modules', name));
const { astro } = iztro('iztro');
const { kot } = iztro('iztro/lib/i18n');

const BRANCHES = ['zi', 'chou', 'yin', 'mao', 'chen', 'si', 'wu', 'wei', 'shen', 'you', 'xu', 'hai'];
const byIndex = (index) => BRANCHES[(index + 2) % 12]; // iztro counts palaces from 寅
const STEM = (name) => kot(name, 'Heavenly').replace('Heavenly', '');
const STARS = {
  ziweiMaj: 'zi_wei', tianjiMaj: 'tian_ji', taiyangMaj: 'tai_yang', wuquMaj: 'wu_qu',
  tiantongMaj: 'tian_tong', lianzhenMaj: 'lian_zhen', tianfuMaj: 'tian_fu', taiyinMaj: 'tai_yin',
  tanlangMaj: 'tan_lang', jumenMaj: 'ju_men', tianxiangMaj: 'tian_xiang', tianliangMaj: 'tian_liang',
  qishaMaj: 'qi_sha', pojunMaj: 'po_jun', zuofuMin: 'zuo_fu', youbiMin: 'you_bi',
  wenchangMin: 'wen_chang', wenquMin: 'wen_qu',
};

const cases = [];
let t = Date.UTC(1955, 2, 7, 3, 30);
while (cases.length < 80) {
  const local = new Date(t + 8 * 3600e3);
  const hour = local.getUTCHours();
  // Skip the late Zi hour (iztro counts it with the next day). The date read falls up to 82
  // years after the birth.
  if (hour !== 23) {
    const target = t + (30 + ((cases.length * 977) % 30000)) * 86400e3;
    const date = `${local.getUTCFullYear()}-${local.getUTCMonth() + 1}-${local.getUTCDate()}`;
    const timeIndex = Math.floor((hour + 1) / 2) % 12;
    const gender = cases.length % 2 ? 'female' : 'male';
    const chart = astro.bySolar(date, timeIndex, gender, true, 'zh-CN');
    const day = new Date(target);
    const iso = day.toISOString().slice(0, 10);
    const h = chart.horoscope(new Date(Date.UTC(day.getUTCFullYear(), day.getUTCMonth(), day.getUTCDate(), 4)), 6);
    const scope = (s) => ({
      branch: byIndex(s.index),
      stem: STEM(s.heavenlyStem),
      transformations: s.mutagen.map((name) => STARS[kot(name)]),
    });
    cases.push({
      utc: new Date(t).toISOString().replace('.000Z', 'Z'),
      sex: gender,
      date: iso,
      nominal_age: h.age.nominalAge,
      childhood: h.decadal.name === '童限',
      decade: scope(h.decadal),
      small_limit: byIndex(h.age.index),
      yearly: scope(h.yearly),
      monthly: scope(h.monthly),
      daily: scope(h.daily),
    });
  }
  t += (311 * 24 + 5) * 3600e3;
}
process.stdout.write('[\n' + cases.map((c) => JSON.stringify(c)).join(',\n') + '\n]\n');
