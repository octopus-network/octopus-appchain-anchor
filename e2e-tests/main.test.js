let anchor, registry, oct;
let anchorName, registryName, octName;

jest.setTimeout(30000);
beforeAll(async function () {
  const near = await nearlib.connect(nearConfig);
  anchorName = nearConfig.anchorName;
  registryName = nearConfig.registryName;
  octName = nearConfig.octName;

  anchor = window.anchor = await near.loadContract(anchorName, {
    viewMethods: ['get_owner'],
    changeMethods: ['new', 'set_owner'],
    sender: anchorName,
  });
  registry = window.registry = await near.loadContract(
    registryName,
    {
      viewMethods: [],
      changeMethods: ['new'],
      sender: registryName,
    }
  );
  oct = window.oct = await near.loadContract(octName, {
    viewMethods: [],
    changeMethods: ['new'],
    sender: anchorName,
  });
});

test('test init', async () => {
  await oct.new({
    owner_id: octName,
    total_supply: '100000000000000000000000000',
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
  const owner = await anchor.get_owner();
  console.log('owner', owner);
  expect(owner).toEqual(anchorName);
});
