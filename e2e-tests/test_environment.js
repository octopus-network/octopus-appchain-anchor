const NodeEnvironment = require('jest-environment-node');
const nearAPI = require('near-api-js');
const fs = require('fs');

const PROJECT_KEY_DIR = './e2e-tests/nearkeys';
// const { PROJECT_KEY_DIR } = require("near-cli//middleware/key-store");

const INITIAL_BALANCE = '500000000000000000000000000';
const testAccountName = 'test.near';

class LocalTestEnvironment extends NodeEnvironment {
  constructor(config) {
    super(config);
  }

  async setup() {
    this.global.nearlib = require('near-api-js');
    this.global.nearAPI = require('near-api-js');
    this.global.window = {};
    let config = require('./get-config')();
    await this.deployContract('anchorName', config, 'appchain_anchor.wasm');
    await this.deployContract(
      'registryName',
      config,
      'mock_appchain_registry.wasm'
    );
    await this.deployContract('octName', config, 'mock_oct_token.wasm');

    await super.setup();
  }

  async deployContract(key, config, fileName) {
    this.global.testSettings = this.global.nearConfig = config;
    const now = Date.now();
    // create random number with at least 7 digits
    const randomNumber = Math.floor(
      Math.random() * (9999999 - 1000000) + 1000000
    );

    const contractName = config[key] + '-' + now + '-' + randomNumber;
    config = Object.assign(config, {
      [key]: contractName,
    });
    const keyStore = new nearAPI.keyStores.UnencryptedFileSystemKeyStore(
      PROJECT_KEY_DIR
    );
    config.deps = Object.assign(config.deps || {}, {
      storage: this.createFakeStorage(),
      keyStore,
    });
    const near = await nearAPI.connect(config);

    const masterAccount = await near.account(testAccountName);
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

  async teardown() {
    await super.teardown();
  }

  runScript(script) {
    return super.runScript(script);
  }

  createFakeStorage() {
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
}

module.exports = LocalTestEnvironment;
