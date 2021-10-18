const { toOctValue, toPrice, toOctValuePrice, toYocto } = utils;
const CALL_GAS = '300000000000000';
const USD_DECIMALS_VALUE = 1_000_000;

let anchor, registry, oct;
let anchorName, registryName, octName;
const vlds = [];
const dlgs = [];

async function latestStakingHistory() {
  const indexRange = await anchor.get_index_range_of_staking_history();
  return await anchor.get_staking_history({
    index: indexRange.end_index,
  });
}

async function stake(caller, amount) {
  const testValidatorIdInAppchain = new Array(64)
    .fill(1)
    .map(() => Math.floor(Math.random() * 16).toString(16))
    .join('');
  await caller.oct.ft_transfer_call(
    {
      receiver_id: anchorName,
      amount,
      msg: JSON.stringify({
        RegisterValidator: {
          validator_id_in_appchain: testValidatorIdInAppchain,
          can_be_delegated_to: true,
        },
      }),
    },
    CALL_GAS,
    '1'
  );
  return testValidatorIdInAppchain;
}

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
    total_supply: toOctValue(100_000_000),
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

  await Promise.all(
    new Array(2).fill(2).map(async (a) => {
      vlds.push(await generateUser(near, 0));
    })
  );
  dlgs.push(await generateUser(near, 0));

  await oct.storage_deposit({ account_id: anchorName }, CALL_GAS, toYocto('1'));
  await Promise.all(
    [...vlds, ...dlgs].map(async (user) => {
      await oct.storage_deposit(
        { account_id: user.accountId },
        CALL_GAS,
        toYocto('1')
      );
      await oct.ft_transfer(
        {
          receiver_id: user.accountId,
          amount: toOctValue('100000'),
        },
        CALL_GAS,
        '1'
      );
      const balance = await user.oct.ft_balance_of({
        account_id: user.accountId,
      });
    })
  );
});

test('test init', async () => {
  const owner = await anchor.get_owner();
  expect(owner).toEqual(masterAccount.accountId);
});

test('test protocol_settings', async () => {
  const wantedProtocolSettings = {
    minimum_validator_deposit: toOctValue('1250'),
    minimum_delegator_deposit: toOctValue('25'),
    minimum_total_stake_price_for_booting: toOctValuePrice('3000'),
    maximum_market_value_percent_of_near_fungible_tokens: 40,
    maximum_market_value_percent_of_wrapped_appchain_token: 45,
    minimum_validator_count: '1',
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
    era_reward: toOctValue('1.2'),
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

test('test set oct price', async () => {
  await anchor.set_price_of_oct_token({ price: toPrice(3) });
});

test('test staking', async () => {
  const appchainState = await anchor.get_appchain_state();
  expect(appchainState).toEqual('Staging');
  const testValidatorIdInAppchain0 = await stake(vlds[0], toOctValue('5000'));
  const testValidatorIdInAppchain1 = await stake(vlds[1], toOctValue('5000'));
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('10000'));
  expect(anchorStatus.validator_count_in_next_era).toEqual('2');
  expect(stakingHistory.staking_fact).toEqual({
    ValidatorRegistered: {
      validator_id: vlds[1].accountId,
      validator_id_in_appchain: testValidatorIdInAppchain1,
      amount: toOctValue('5000'),
      can_be_delegated_to: true,
    },
  });
});

test('test increase stake', async () => {
  await vlds[0].oct.ft_transfer_call(
    {
      receiver_id: anchorName,
      amount: toOctValue('50'),
      msg: '"IncreaseStake"',
    },
    CALL_GAS,
    '1'
  );
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('10050'));
  expect(stakingHistory.staking_fact).toEqual({
    StakeIncreased: {
      validator_id: vlds[0].accountId,
      amount: toOctValue('50'),
    },
  });
});

test('test go booting', async () => {
  await anchor.go_booting();
  const appchainState = await anchor.get_appchain_state();
  expect(appchainState).toEqual('Booting');
});

test('test go live', async () => {
  await anchor.go_live();
  const appchainState = await anchor.get_appchain_state();
  expect(appchainState).toEqual('Active');
});

test('test decrease stake', async () => {
  await vlds[0].anchor.decrease_stake({
    amount: toOctValue('50'),
  });
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('10000'));
  expect(stakingHistory.staking_fact).toEqual({
    StakeDecreased: {
      validator_id: vlds[0].accountId,
      amount: toOctValue('50'),
    },
  });
});

test('test delegation', async () => {
  await dlgs[0].oct.ft_transfer_call(
    {
      receiver_id: anchorName,
      amount: toOctValue('2000'),
      msg: JSON.stringify({
        RegisterDelegator: {
          validator_id: vlds[1].accountId,
        },
      }),
    },
    CALL_GAS,
    '1'
  );
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();

  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('12000'));
  expect(stakingHistory.staking_fact).toEqual({
    DelegatorRegistered: {
      delegator_id: dlgs[0].accountId,
      validator_id: vlds[1].accountId,
      amount: toOctValue('2000'),
    },
  });
});

test('test increase delegation', async () => {
  await dlgs[0].oct.ft_transfer_call(
    {
      receiver_id: anchorName,
      amount: toOctValue('300'),
      msg: JSON.stringify({
        IncreaseDelegation: {
          validator_id: vlds[1].accountId,
        },
      }),
    },
    CALL_GAS,
    '1'
  );
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('12300'));
  expect(stakingHistory.staking_fact).toEqual({
    DelegationIncreased: {
      delegator_id: dlgs[0].accountId,
      validator_id: vlds[1].accountId,
      amount: toOctValue('300'),
    },
  });
});

test('test decrease delegation', async () => {
  await dlgs[0].anchor.decrease_delegation({
    validator_id: vlds[1].accountId,
    amount: toOctValue('300'),
  });
  const anchorStatus = await anchor.get_anchor_status();
  const stakingHistory = await latestStakingHistory();
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('12000'));
  expect(stakingHistory.staking_fact).toEqual({
    DelegationDecreased: {
      delegator_id: dlgs[0].accountId,
      validator_id: vlds[1].accountId,
      amount: toOctValue('300'),
    },
  });
});

test('test unbond validator', async () => {
  await vlds[0].anchor.unbond_stake();
  const anchorStatus = await anchor.get_anchor_status();
  const unbondedStakes = await anchor.get_unbonded_stakes_of({
    account_id: masterAccount.accountId,
  });
  const stakingHistory = await latestStakingHistory();
  console.log('unbondedStakes', unbondedStakes);
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('7000'));
  expect(stakingHistory.staking_fact).toEqual({
    ValidatorUnbonded: {
      validator_id: vlds[0].accountId,
      amount: toOctValue('5000'),
    },
  });
});

test('test unbond delegator', async () => {
  await dlgs[0].anchor.unbond_delegation({
    validator_id: vlds[1].accountId,
  });
  const anchorStatus = await anchor.get_anchor_status();
  const unbondedStakes = await anchor.get_unbonded_stakes_of({
    account_id: masterAccount.accountId,
  });
  const stakingHistory = await latestStakingHistory();
  console.log('unbondedStakes', unbondedStakes);
  expect(anchorStatus.total_stake_in_next_era).toEqual(toOctValue('5000'));
  expect(stakingHistory.staking_fact).toEqual({
    DelegatorUnbonded: {
      delegator_id: dlgs[0].accountId,
      validator_id: vlds[1].accountId,
      amount: toOctValue('2000'),
    },
  });
});

// test('test withdraw stake for validator', async () => {
//   const balanceBefore = await vlds[0].oct.ft_balance_of({
//     account_id: vlds[0].accountId,
//   });
//   await vlds[0].anchor.withdraw_stake({ account_id: vlds[0].accountId });
//   const balanceAfter = await vlds[0].oct.ft_balance_of({
//     account_id: vlds[0].accountId,
//   });
//   console.log('balanceBefore', balanceBefore);
//   console.log('balanceAfter', balanceAfter);
// });

// test('test withdraw stake for delegator', async () => {
//   const balanceBefore = await vlds[0].oct.ft_balance_of({
//     account_id: dlgs[0].accountId,
//   });
//   await dlgs[0].anchor.withdraw_stake({ account_id: dlgs[0].accountId });
//   const balanceAfter = await dlgs[0].oct.ft_balance_of({
//     account_id: dlgs[0].accountId,
//   });
//   console.log('balanceBefore', balanceBefore);
//   console.log('balanceAfter', balanceAfter);
// });
