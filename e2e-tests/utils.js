const BigNumber = require('bignumber.js');
BigNumber.config({ EXPONENTIAL_AT: 30 });
const utils = {
  toOctValue: (origin) =>
    new BigNumber(origin)
      .times(new BigNumber('1000000000000000000'))
      .toString(),
  toPrice: (origin) =>
    new BigNumber(origin).times(new BigNumber('1000000')).toString(),
  toYocto: (origin) =>
    new BigNumber(origin)
      .times(new BigNumber('1000000000000000000000000'))
      .toString(),
};

module.exports = utils;
