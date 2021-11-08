const NodeEnvironment = require('jest-environment-node');
const nearAPI = require('near-api-js');
const fs = require('fs');
const util = require('util');
const readFile = util.promisify(fs.readFile);
let near, masterAccount;
const PROJECT_KEY_DIR = './e2e-tests/nearkeys';

const INITIAL_BALANCE = '10000000000000000000000000';
const keyStore = new nearAPI.keyStores.UnencryptedFileSystemKeyStore(
  PROJECT_KEY_DIR
);

const config = require('./get-config')();
config.deps = Object.assign(config.deps || {}, {
  storage: createFakeStorage(),
  keyStore,
});

class LocalTestEnvironment extends NodeEnvironment {
  constructor(config) {
    super(config);
  }

  async setup() {
    near = await nearAPI.connect(config);
    masterAccount = await near.account(config.masterAccount);

    this.global.testSettings = this.global.nearConfig = config;
    this.global.nearlib = require('near-api-js');
    this.global.nearAPI = require('near-api-js');
    this.global.window = {};
    this.global.anchorMethods = require('./anchor-methods');
    this.global.octMethods = require('./oct-methods');
    this.global.utils = require('./utils');
    this.global.masterAccount = masterAccount;

    await this.deployContract('anchorName', config, 'appchain_anchor.wasm');
    await this.deployContract(
      'registryName',
      config,
      'mock_appchain_registry.wasm'
    );
    await this.deployContract('octName', config, 'mock_oct_token.wasm');
    await this.deployContract('wrappedAppchainToken', config, 'mock_wrapped_appchain_token.wasm');

    await super.setup();

    this.global.createUser = async function (accountId) {
      const now = Date.now();
      const randomNumber = Math.floor(
        Math.random() * (9999999 - 1000000) + 1000000
      );
      const randomKey = await nearAPI.KeyPair.fromRandom('ed25519');
      const newAccountId = accountId + '-' + now + '-' + randomNumber + '.' + config.masterAccount;
      await masterAccount.createAccount(
        newAccountId,
        randomKey.getPublicKey(),
        INITIAL_BALANCE
      );
      keyStore.setKey(config.networkId, newAccountId, randomKey);
      return new nearAPI.Account(near.connection, newAccountId);
    };

    this.global.generateUser = async (near, index) => {
      const account = await this.global.createUser(`${index}`);
      const accountId = account.accountId;
      const user = {
        accountId,
        oct: await near.loadContract(config.octName, {
          ...this.global.octMethods,
          sender: accountId,
        }),
        anchor: await near.loadContract(config.anchorName, {
          ...this.global.anchorMethods,
          sender: accountId,
        }),
        appchianToken: await near.loadContract(config.wrappedAppchainToken, {
          ...this.global.octMethods,
          sender: accountId,
        }),
      };
      return user;
    };
  }

  async deployContract(key, config, fileName) {
    const now = Date.now();
    // create random number with at least 7 digits
    const randomNumber = Math.floor(
      Math.random() * (9999999 - 1000000) + 1000000
    );
    const contractName = config[key] + '-' + now + '-' + randomNumber;
    config = Object.assign(config, {
      [key]: contractName,
    });
    const randomKey = await nearAPI.KeyPair.fromRandom('ed25519');
    const data = [
      ...fs.readFileSync(`./res/${fileName}`),
    ];
    await config.deps.keyStore.setKey(
      config.networkId,
      contractName,
      randomKey
    );
    await masterAccount.createAndDeployContract(
      contractName,
      randomKey.getPublicKey(),
      data,
      INITIAL_BALANCE
    );
  }

  async createContractUser(account, contractAccountId, contractMethods) {
    const accountUseContract = new nearAPI.Contract(
      account,
      contractAccountId,
      contractMethods
    );
    return accountUseContract;
  }

  async teardown() {
    await super.teardown();
  }

  runScript(script) {
    return super.runScript(script);
  }
}
function createFakeStorage() {
  let store = {};
  return {
    getItem: function (key) {
      return store[key];
    },
    setItem: function (key, value) {
      store[key] = value.toString();
    },
    clear: function () {
      store = {};
    },
    removeItem: function (key) {
      delete store[key];
    },
  };
}

module.exports = LocalTestEnvironment;
