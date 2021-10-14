const { octToBnStr, toYocto } = utils;
const CALL_GAS = '300000000000000';

let anchor, registry, oct;
let anchorName, registryName, octName;

jest.setTimeout(50000);
beforeAll(async function () {
  const near = await nearlib.connect(nearConfig);
  anchorName = nearConfig.anchorName;
  registryName = nearConfig.registryName;
  octName = nearConfig.octName;

  anchor = window.anchor = await near.loadContract(anchorName, {
    ...anchorMethods,
    sender: masterAccount.accountId,
  });
  registry = window.registry = await near.loadContract(registryName, {
    viewMethods: [],
    changeMethods: ['new'],
    sender: masterAccount.accountId,
  });
  oct = window.oct = await near.loadContract(octName, {
    ...octMethods,
    sender: masterAccount.accountId,
  });

  await oct.new({
    owner_id: masterAccount.accountId,
    total_supply: octToBnStr(100_000_000),
    metadata: {
      spec: 'ft-1.0.0',
      name: 'OCT_TEST',
      symbol: 'OCT',
      decimals: 18,
    },
  });
  await registry.new({
    owner: registryName,
    oct_token: octName,
  });
  await anchor.new({
    appchain_id: 'testAppchain',
    appchain_registry: registryName,
    oct_token: octName,
  });

  const alice = await createUser('alice');
  const aliceOct = await near.loadContract(octName, {
    ...octMethods,
    sender: alice.accountId,
  });
  const aliceAnchor = await near.loadContract(anchorName, {
    ...anchorMethods,
    sender: alice.accountId,
  });

  await oct.storage_deposit({ account_id: anchorName }, CALL_GAS, toYocto('1'));
  await oct.storage_deposit(
    { account_id: alice.accountId },
    CALL_GAS,
    toYocto('1')
  );
});

test('test init', async () => {
  const owner = await anchor.get_owner();
  expect(owner).toEqual(masterAccount.accountId);
});

test('test protocol_settings', async () => {
  const wantedProtocolSettings = {
    minimum_validator_deposit: octToBnStr('1250'),
    minimum_delegator_deposit: octToBnStr('25'),
    minimum_total_stake_price_for_booting: octToBnStr('3000'),
    maximum_market_value_percent_of_near_fungible_tokens: 40,
    maximum_market_value_percent_of_wrapped_appchain_token: 45,
    minimum_validator_count: '3',
    maximum_validators_per_delegator: '2',
    unlock_period_of_validator_deposit: '10',
    unlock_period_of_delegator_deposit: '8',
    maximum_era_count_of_unwithdrawn_reward: '70',
    maximum_era_count_of_valid_appchain_message: '5',
    delegation_fee_percent: 10,
  };
  await Promise.all[
    (await anchor.change_minimum_validator_deposit({
      value: wantedProtocolSettings.minimum_validator_deposit,
    }),
    await anchor.change_minimum_validator_deposit({
      value: wantedProtocolSettings.minimum_validator_deposit,
    }),
    await anchor.change_minimum_delegator_deposit({
      value: wantedProtocolSettings.minimum_delegator_deposit,
    }),
    await anchor.change_minimum_total_stake_price_for_booting({
      value: wantedProtocolSettings.minimum_total_stake_price_for_booting,
    }),
    await anchor.change_maximum_market_value_percent_of_near_fungible_tokens({
      value:
        wantedProtocolSettings.maximum_market_value_percent_of_near_fungible_tokens,
    }),
    await anchor.change_maximum_market_value_percent_of_wrapped_appchain_token({
      value:
        wantedProtocolSettings.maximum_market_value_percent_of_wrapped_appchain_token,
    }),
    await anchor.change_minimum_validator_count({
      value: wantedProtocolSettings.minimum_validator_count,
    }),
    await anchor.change_maximum_validators_per_delegator({
      value: wantedProtocolSettings.maximum_validators_per_delegator,
    }),
    await anchor.change_unlock_period_of_validator_deposit({
      value: wantedProtocolSettings.unlock_period_of_validator_deposit,
    }),
    await anchor.change_unlock_period_of_delegator_deposit({
      value: wantedProtocolSettings.unlock_period_of_delegator_deposit,
    }),
    await anchor.change_maximum_era_count_of_unwithdrawn_reward({
      value: wantedProtocolSettings.maximum_era_count_of_unwithdrawn_reward,
    }),
    await anchor.change_maximum_era_count_of_valid_appchain_message({
      value: wantedProtocolSettings.maximum_era_count_of_valid_appchain_message,
    }),
    await anchor.change_delegation_fee_percent({
      value: wantedProtocolSettings.delegation_fee_percent,
    }))
  ];
  const newProtocolSettings = await anchor.get_protocol_settings();
  expect(newProtocolSettings).toEqual(wantedProtocolSettings);
});

test('test appchain_settings', async () => {
  const wantedAppchainSetting = {
    chain_spec: 'chain_spec_url_for_test',
    raw_chain_spec: 'raw_chain_spec_url_for_test',
    boot_nodes: `["/ip4/3.113.45.140/tcp/30333/p2p/12D3KooWAxYKgdmTczLioD1jkzMyaDuV2Q5VHBsJxPr5zEmHr8nY",   "/ip4/18.179.183.182/tcp/30333/p2p/12D3KooWSmLVShww4w9PVW17cCAS5C1JnXBU4NbY7FcGGjMyUGiq",   "/ip4/54.168.14.201/tcp/30333/p2p/12D3KooWT2umkS7F8GzUTLrfUzVBJPKn6YwCcuv6LBFQ27UPoo2Y",   "/ip4/35.74.18.116/tcp/30333/p2p/12D3KooWHNf9JxUZKHoF7rrsmorv86gonXSb2ZU44CbMsnBNFSAJ", ]`,
    rpc_endpoint: 'wss://test.rpc.testnet.oct.network:9944',
    era_reward: octToBnStr('1.2'),
  };
  await Promise.all[
    (await anchor.set_chain_spec({
      chain_spec: wantedAppchainSetting.chain_spec,
    }),
    await anchor.set_raw_chain_spec({
      raw_chain_spec: wantedAppchainSetting.raw_chain_spec,
    }),
    await anchor.set_boot_nodes({
      boot_nodes: wantedAppchainSetting.boot_nodes,
    }),
    await anchor.set_rpc_endpoint({
      rpc_endpoint: wantedAppchainSetting.rpc_endpoint,
    }),
    await anchor.set_era_reward({
      era_reward: wantedAppchainSetting.era_reward,
    }))
  ];
  const newAnchorSettings = await anchor.get_appchain_settings();
  expect(newAnchorSettings).toEqual(wantedAppchainSetting);
});

test('test anchor_settings', async () => {
  const wantedAnchorSetting = {
    token_price_maintainer_account: masterAccount.accountId,
  };
  await anchor.set_token_price_maintainer_account({
    account_id: wantedAnchorSetting.token_price_maintainer_account,
  });
  const newAnchorSettings = await anchor.get_anchor_settings();
  expect(newAnchorSettings).toEqual(wantedAnchorSetting);
});

test('test staking ', async () => {
  const appchainState = await anchor.get_appchain_state();
  expect(appchainState).toEqual('Staging');
  await oct.ft_transfer_call(
    {
      receiver_id: anchorName,
      amount: octToBnStr('10000'),
      msg: JSON.stringify({
        RegisterValidator: {
          validator_id_in_appchain:
            'c425bbf59c7bf49e4fcc6547539d84ba8ecd2fb171f5b83cde3571d45d0c8224',
          can_be_delegated_to: true,
        },
      }),
    },
    CALL_GAS,
    '1'
  );
  const anchorStatus = await anchor.get_anchor_status();
  expect(anchorStatus.total_stake_in_next_era).toEqual(octToBnStr('10000'));
  expect(anchorStatus.validator_count_in_next_era).toEqual('1');
});
