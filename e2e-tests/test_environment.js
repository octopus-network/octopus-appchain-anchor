const NodeEnvironment = require('jest-environment-node');
const nearAPI = require('near-api-js');
const fs = require('fs');
let near, masterAccount;
const PROJECT_KEY_DIR = './e2e-tests/nearkeys';
// const { PROJECT_KEY_DIR } = require("near-cli//middleware/key-store");

const INITIAL_BALANCE = '8000000000000000000000000';
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
    const keyFile = require(config.keyPath);
    const masterKey = nearAPI.utils.KeyPair.fromString(
      keyFile.secret_key || keyFile.private_key
    );

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

    await super.setup();

    this.global.createUser = async function (accountId) {
      const now = Date.now();
      const randomNumber = Math.floor(
        Math.random() * (9999999 - 1000000) + 1000000
      );
      const randomKey = await nearAPI.KeyPair.fromRandom('ed25519');
      await masterAccount.createAccount(
        accountId + '-' + now + '-' + randomNumber,
        masterKey.getPublicKey(),
        INITIAL_BALANCE
      );
      keyStore.setKey(config.networkId, accountId, randomKey);
      return new nearAPI.Account(near.connection, accountId);
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
      ...fs.readFileSync(`./target/wasm32-unknown-unknown/release/${fileName}`),
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
