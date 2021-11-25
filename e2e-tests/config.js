const { ANCHOR_NAME, REGISTRY_NAME, OCT_NAME, WRAPPED_APPCHAIN_TOKEN } = process.env;

function getConfig(env) {
  switch (env) {
    case 'production':
    case 'mainnet':
      return {
        networkId: 'mainnet',
        nodeUrl: 'https://rpc.mainnet.near.org',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        walletUrl: 'https://wallet.near.org',
        helperUrl: 'https://helper.mainnet.near.org',
        explorerUrl: 'https://explorer.mainnet.near.org',
      };
    case 'development':
    case 'testnet':
      return {
        networkId: 'testnet',
        nodeUrl: 'https://rpc.testnet.near.org',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        walletUrl: 'https://wallet.testnet.near.org',
        helperUrl: 'https://helper.testnet.near.org',
        explorerUrl: 'https://explorer.testnet.near.org',
        masterAccount: 'e2e-test.testnet',
      };
    case 'betanet':
      return {
        networkId: 'betanet',
        nodeUrl: 'https://rpc.betanet.near.org',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        walletUrl: 'https://wallet.betanet.near.org',
        helperUrl: 'https://helper.betanet.near.org',
        explorerUrl: 'https://explorer.betanet.near.org',
      };
    case 'local':
      return {
        networkId: 'sandbox',
        nodeUrl: 'http://localhost:3030',
        keyPath: `${process.env.HOME}/.near/localnet/node0/validator_key.json`,
        walletUrl: 'http://localhost:4000/wallet',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        masterAccount: "test.near",
        keyPath: "/tmp/near-sandbox/validator_key.json",
      };
    case 'test':
    case 'ci':
      return {
        networkId: 'shared-test',
        nodeUrl: 'https://rpc.ci-testnet.near.org',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        masterAccount: 'test.near',
      };
    case 'ci-betanet':
      return {
        networkId: 'shared-test-staging',
        nodeUrl: 'https://rpc.ci-betanet.near.org',
        anchorName: ANCHOR_NAME,
        registryName: REGISTRY_NAME,
        octName: OCT_NAME,
        wrappedAppchainToken: WRAPPED_APPCHAIN_TOKEN,
        masterAccount: 'test.near',
      };
    default:
      throw Error(
        `Unconfigured environment '${env}'. Can be configured in src/config.js.`
      );
  }
}

module.exports = getConfig;
