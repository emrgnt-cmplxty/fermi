import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', 'a8e'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', '2c0'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', '4bd'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', '28f'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '404'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', 'e95'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '43d'),
    exact: true
  },
  {
    path: '/search',
    component: ComponentCreator('/search', 'f11'),
    exact: true
  },
  {
    path: '/tools/near-api-js/reference',
    component: ComponentCreator('/tools/near-api-js/reference', '27a'),
    routes: [
      {
        path: '/tools/near-api-js/reference/dummy',
        component: ComponentCreator('/tools/near-api-js/reference/dummy', '00b'),
        exact: true
      }
    ]
  },
  {
    path: '/tools/near-sdk-js/reference',
    component: ComponentCreator('/tools/near-sdk-js/reference', '558'),
    routes: [
      {
        path: '/tools/near-sdk-js/reference/dummy',
        component: ComponentCreator('/tools/near-sdk-js/reference/dummy', 'b53'),
        exact: true
      }
    ]
  },
  {
    path: '/',
    component: ComponentCreator('/', '9f7'),
    routes: [
      {
        path: '/',
        component: ComponentCreator('/', '5cd'),
        exact: true
      },
      {
        path: '/api/rpc/access-keys',
        component: ComponentCreator('/api/rpc/access-keys', '4e0'),
        exact: true
      },
      {
        path: '/api/rpc/block-info',
        component: ComponentCreator('/api/rpc/block-info', 'e8c'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/contracts',
        component: ComponentCreator('/api/rpc/contracts', 'a0c'),
        exact: true
      },
      {
        path: '/api/rpc/futures-markets',
        component: ComponentCreator('/api/rpc/futures-markets', '178'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/gas',
        component: ComponentCreator('/api/rpc/gas', '865'),
        exact: true
      },
      {
        path: '/api/rpc/introduction',
        component: ComponentCreator('/api/rpc/introduction', '715'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/network',
        component: ComponentCreator('/api/rpc/network', '4c8'),
        exact: true
      },
      {
        path: '/api/rpc/protocol',
        component: ComponentCreator('/api/rpc/protocol', 'b32'),
        exact: true
      },
      {
        path: '/api/rpc/providers',
        component: ComponentCreator('/api/rpc/providers', 'a10'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/setup',
        component: ComponentCreator('/api/rpc/setup', 'a18'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/transactions',
        component: ComponentCreator('/api/rpc/transactions', 'fff'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/concepts/advanced/papers',
        component: ComponentCreator('/concepts/advanced/papers', '3aa'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/advanced/specification',
        component: ComponentCreator('/concepts/advanced/specification', '6af'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/advanced/vm',
        component: ComponentCreator('/concepts/advanced/vm', '27e'),
        exact: true
      },
      {
        path: '/concepts/basics/accounts/access-keys',
        component: ComponentCreator('/concepts/basics/accounts/access-keys', 'a4f'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/accounts/account-id',
        component: ComponentCreator('/concepts/basics/accounts/account-id', 'a3b'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/accounts/creating-accounts',
        component: ComponentCreator('/concepts/basics/accounts/creating-accounts', '440'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/accounts/model',
        component: ComponentCreator('/concepts/basics/accounts/model', 'b1c'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/accounts/smartcontract',
        component: ComponentCreator('/concepts/basics/accounts/smartcontract', '69e'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/accounts/state',
        component: ComponentCreator('/concepts/basics/accounts/state', 'e47'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/actors',
        component: ComponentCreator('/concepts/basics/actors', '06d'),
        exact: true
      },
      {
        path: '/concepts/basics/epoch',
        component: ComponentCreator('/concepts/basics/epoch', '186'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/networks',
        component: ComponentCreator('/concepts/basics/networks', '3d5'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/overview',
        component: ComponentCreator('/concepts/basics/overview', '50a'),
        exact: true
      },
      {
        path: '/concepts/basics/protocol',
        component: ComponentCreator('/concepts/basics/protocol', 'db4'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/runtime',
        component: ComponentCreator('/concepts/basics/runtime', '866'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/technical_stuff',
        component: ComponentCreator('/concepts/basics/technical_stuff', 'a4f'),
        exact: true
      },
      {
        path: '/concepts/basics/token-loss',
        component: ComponentCreator('/concepts/basics/token-loss', '30c'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/tokens',
        component: ComponentCreator('/concepts/basics/tokens', 'ac8'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/transactions/gas',
        component: ComponentCreator('/concepts/basics/transactions/gas', '37f'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/transactions/gas-advanced',
        component: ComponentCreator('/concepts/basics/transactions/gas-advanced', 'f99'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/transactions/overview',
        component: ComponentCreator('/concepts/basics/transactions/overview', 'a6b'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/basics/validators',
        component: ComponentCreator('/concepts/basics/validators', 'e64'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/storage/data-storage',
        component: ComponentCreator('/concepts/storage/data-storage', '36c'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/storage/storage-solutions',
        component: ComponentCreator('/concepts/storage/storage-solutions', '2e7'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/storage/storage-staking',
        component: ComponentCreator('/concepts/storage/storage-staking', 'd7f'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/web3/basics',
        component: ComponentCreator('/concepts/web3/basics', 'faf'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/web3/economics',
        component: ComponentCreator('/concepts/web3/economics', '973'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/web3/intro',
        component: ComponentCreator('/concepts/web3/intro', 'afe'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/web3/near',
        component: ComponentCreator('/concepts/web3/near', 'd3e'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/web3/nfts',
        component: ComponentCreator('/concepts/web3/nfts', '3e2'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/concepts/welcome',
        component: ComponentCreator('/concepts/welcome', '026'),
        exact: true,
        sidebar: "concepts"
      },
      {
        path: '/develop/contracts/actions',
        component: ComponentCreator('/develop/contracts/actions', '6e2'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/anatomy',
        component: ComponentCreator('/develop/contracts/anatomy', '2dd'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/crosscontract',
        component: ComponentCreator('/develop/contracts/crosscontract', 'ca8'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/environment/',
        component: ComponentCreator('/develop/contracts/environment/', 'efc'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/environment/table.as',
        component: ComponentCreator('/develop/contracts/environment/table.as', '7a8'),
        exact: true
      },
      {
        path: '/develop/contracts/environment/table.js',
        component: ComponentCreator('/develop/contracts/environment/table.js', '0ea'),
        exact: true
      },
      {
        path: '/develop/contracts/environment/table.rs',
        component: ComponentCreator('/develop/contracts/environment/table.rs', 'f24'),
        exact: true
      },
      {
        path: '/develop/contracts/introduction',
        component: ComponentCreator('/develop/contracts/introduction', '109'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/callbacks',
        component: ComponentCreator('/develop/contracts/security/callbacks', 'e5b'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/checklist',
        component: ComponentCreator('/develop/contracts/security/checklist', 'e6b'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/frontrunning',
        component: ComponentCreator('/develop/contracts/security/frontrunning', '593'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/one-yocto',
        component: ComponentCreator('/develop/contracts/security/one-yocto', '609'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/random',
        component: ComponentCreator('/develop/contracts/security/random', '2ee'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/reentrancy-attacks',
        component: ComponentCreator('/develop/contracts/security/reentrancy-attacks', '875'),
        exact: true
      },
      {
        path: '/develop/contracts/security/storage',
        component: ComponentCreator('/develop/contracts/security/storage', '26a'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/security/sybil',
        component: ComponentCreator('/develop/contracts/security/sybil', '747'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/standards',
        component: ComponentCreator('/develop/contracts/standards', 'ba8'),
        exact: true
      },
      {
        path: '/develop/contracts/storage',
        component: ComponentCreator('/develop/contracts/storage', 'eed'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/contracts/whatisacontract',
        component: ComponentCreator('/develop/contracts/whatisacontract', '1a1'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/deploy',
        component: ComponentCreator('/develop/deploy', '12d'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/integrate/cli',
        component: ComponentCreator('/develop/integrate/cli', '59f'),
        exact: true
      },
      {
        path: '/develop/integrate/frontend',
        component: ComponentCreator('/develop/integrate/frontend', '696'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/integrate/rpc',
        component: ComponentCreator('/develop/integrate/rpc', '520'),
        exact: true
      },
      {
        path: '/develop/prerequisites',
        component: ComponentCreator('/develop/prerequisites', '672'),
        exact: true
      },
      {
        path: '/develop/quickstart-guide',
        component: ComponentCreator('/develop/quickstart-guide', 'fca'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/relevant-contracts/dao',
        component: ComponentCreator('/develop/relevant-contracts/dao', '41a'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/develop/relevant-contracts/ft',
        component: ComponentCreator('/develop/relevant-contracts/ft', 'ee5'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/develop/relevant-contracts/nft',
        component: ComponentCreator('/develop/relevant-contracts/nft', '72f'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/develop/relevant-contracts/oracles',
        component: ComponentCreator('/develop/relevant-contracts/oracles', '8e4'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/testing/integration-test',
        component: ComponentCreator('/develop/testing/integration-test', 'e85'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/testing/introduction',
        component: ComponentCreator('/develop/testing/introduction', '251'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/testing/kurtosis-localnet',
        component: ComponentCreator('/develop/testing/kurtosis-localnet', '7b2'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/testing/unit-test',
        component: ComponentCreator('/develop/testing/unit-test', '963'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/testing/workspaces-migration',
        component: ComponentCreator('/develop/testing/workspaces-migration', 'f57'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/upgrade-and-lock',
        component: ComponentCreator('/develop/upgrade-and-lock', '0a8'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/upgrade/dao-updates',
        component: ComponentCreator('/develop/upgrade/dao-updates', 'ee3'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/upgrade/migration',
        component: ComponentCreator('/develop/upgrade/migration', '642'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/develop/welcome',
        component: ComponentCreator('/develop/welcome', '6a0'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/integrator/accounts',
        component: ComponentCreator('/integrator/accounts', '947'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/balance-changes',
        component: ComponentCreator('/integrator/balance-changes', '10e'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/create-transactions',
        component: ComponentCreator('/integrator/create-transactions', '0a8'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/errors/error-implementation',
        component: ComponentCreator('/integrator/errors/error-implementation', '28d'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/errors/introduction',
        component: ComponentCreator('/integrator/errors/introduction', '6ae'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/errors/token-loss',
        component: ComponentCreator('/integrator/errors/token-loss', 'f9d'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/exchange-integration',
        component: ComponentCreator('/integrator/exchange-integration', '2a4'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/faq',
        component: ComponentCreator('/integrator/faq', '41b'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/fungible-tokens',
        component: ComponentCreator('/integrator/fungible-tokens', '125'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/integrator/implicit-accounts',
        component: ComponentCreator('/integrator/implicit-accounts', '448'),
        exact: true,
        sidebar: "integrate"
      },
      {
        path: '/sdk/rust/best-practices',
        component: ComponentCreator('/sdk/rust/best-practices', '9c2'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/building/basics',
        component: ComponentCreator('/sdk/rust/building/basics', '300'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/building/post-processing',
        component: ComponentCreator('/sdk/rust/building/post-processing', 'a94'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/building/prototyping',
        component: ComponentCreator('/sdk/rust/building/prototyping', 'fe6'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/building/reproducible-builds',
        component: ComponentCreator('/sdk/rust/building/reproducible-builds', '89d'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-interface/contract-mutability',
        component: ComponentCreator('/sdk/rust/contract-interface/contract-mutability', '34b'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-interface/payable-methods',
        component: ComponentCreator('/sdk/rust/contract-interface/payable-methods', 'ab5'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-interface/private-methods',
        component: ComponentCreator('/sdk/rust/contract-interface/private-methods', 'b27'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-interface/public-methods',
        component: ComponentCreator('/sdk/rust/contract-interface/public-methods', 'cea'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-interface/serialization-interface',
        component: ComponentCreator('/sdk/rust/contract-interface/serialization-interface', '3f5'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-size',
        component: ComponentCreator('/sdk/rust/contract-size', 'fef'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-structure/collections',
        component: ComponentCreator('/sdk/rust/contract-structure/collections', '4d2'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-structure/near-bindgen',
        component: ComponentCreator('/sdk/rust/contract-structure/near-bindgen', '5ef'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/contract-structure/nesting',
        component: ComponentCreator('/sdk/rust/contract-structure/nesting', '7e5'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/cross-contract/callbacks',
        component: ComponentCreator('/sdk/rust/cross-contract/callbacks', '5c7'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/introduction',
        component: ComponentCreator('/sdk/rust/introduction', 'b9c'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/promises/create-account',
        component: ComponentCreator('/sdk/rust/promises/create-account', '626'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/promises/deploy-contract',
        component: ComponentCreator('/sdk/rust/promises/deploy-contract', 'f98'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/promises/intro',
        component: ComponentCreator('/sdk/rust/promises/intro', '35a'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/promises/token-tx',
        component: ComponentCreator('/sdk/rust/promises/token-tx', '601'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/testing/integration-tests',
        component: ComponentCreator('/sdk/rust/testing/integration-tests', '091'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/rust/testing/unit-tests',
        component: ComponentCreator('/sdk/rust/testing/unit-tests', 'ab2'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/sdk/welcome',
        component: ComponentCreator('/sdk/welcome', '04d'),
        exact: true,
        sidebar: "sdk"
      },
      {
        path: '/tools/explorer',
        component: ComponentCreator('/tools/explorer', '1f6'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/indexer-for-explorer',
        component: ComponentCreator('/tools/indexer-for-explorer', '3b9'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-api-js/account',
        component: ComponentCreator('/tools/near-api-js/account', 'c34'),
        exact: true
      },
      {
        path: '/tools/near-api-js/contract',
        component: ComponentCreator('/tools/near-api-js/contract', '359'),
        exact: true
      },
      {
        path: '/tools/near-api-js/cookbook',
        component: ComponentCreator('/tools/near-api-js/cookbook', '5a2'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-api-js/faq',
        component: ComponentCreator('/tools/near-api-js/faq', 'f76'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-api-js/quick-reference',
        component: ComponentCreator('/tools/near-api-js/quick-reference', '8ac'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-api-js/utils',
        component: ComponentCreator('/tools/near-api-js/utils', '812'),
        exact: true
      },
      {
        path: '/tools/near-api-js/wallet',
        component: ComponentCreator('/tools/near-api-js/wallet', 'ad4'),
        exact: true
      },
      {
        path: '/tools/near-cli',
        component: ComponentCreator('/tools/near-cli', '0a4'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-sdk-js',
        component: ComponentCreator('/tools/near-sdk-js', '608'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/near-sdk-rs',
        component: ComponentCreator('/tools/near-sdk-rs', 'c6a'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/realtime',
        component: ComponentCreator('/tools/realtime', '479'),
        exact: true,
        sidebar: "develop"
      },
      {
        path: '/tools/usecases',
        component: ComponentCreator('/tools/usecases', 'e15'),
        exact: true
      },
      {
        path: '/tools/welcome',
        component: ComponentCreator('/tools/welcome', 'e2c'),
        exact: true
      },
      {
        path: '/tutorials/crosswords/basics/add-functions-call',
        component: ComponentCreator('/tutorials/crosswords/basics/add-functions-call', '3a3'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/basics/hashing-and-unit-tests',
        component: ComponentCreator('/tutorials/crosswords/basics/hashing-and-unit-tests', 'a18'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/basics/overview',
        component: ComponentCreator('/tutorials/crosswords/basics/overview', '03e'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/basics/set-up-skeleton',
        component: ComponentCreator('/tutorials/crosswords/basics/set-up-skeleton', '920'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/basics/simple-frontend',
        component: ComponentCreator('/tutorials/crosswords/basics/simple-frontend', '20a'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/actions',
        component: ComponentCreator('/tutorials/crosswords/beginner/actions', '328'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/adding-a-puzzle',
        component: ComponentCreator('/tutorials/crosswords/beginner/adding-a-puzzle', 'd14'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/collections',
        component: ComponentCreator('/tutorials/crosswords/beginner/collections', 'd58'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/logging-in',
        component: ComponentCreator('/tutorials/crosswords/beginner/logging-in', '452'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/logging-in-implementation',
        component: ComponentCreator('/tutorials/crosswords/beginner/logging-in-implementation', 'bb6'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/overview',
        component: ComponentCreator('/tutorials/crosswords/beginner/overview', '03b'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/beginner/structs-enums',
        component: ComponentCreator('/tutorials/crosswords/beginner/structs-enums', '07d'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/access-key-solution',
        component: ComponentCreator('/tutorials/crosswords/intermediate/access-key-solution', '765'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/base64vecu8',
        component: ComponentCreator('/tutorials/crosswords/intermediate/base64vecu8', 'f85'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/cross-contract-calls',
        component: ComponentCreator('/tutorials/crosswords/intermediate/cross-contract-calls', 'b04'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/linkdrop',
        component: ComponentCreator('/tutorials/crosswords/intermediate/linkdrop', '3e2'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/overview',
        component: ComponentCreator('/tutorials/crosswords/intermediate/overview', '49c'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/crosswords/intermediate/use-seed-phrase',
        component: ComponentCreator('/tutorials/crosswords/intermediate/use-seed-phrase', 'e38'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/coin-flip',
        component: ComponentCreator('/tutorials/examples/coin-flip', 'ba2'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/count-near',
        component: ComponentCreator('/tutorials/examples/count-near', 'cd2'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/donation',
        component: ComponentCreator('/tutorials/examples/donation', '9b7'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/guest-book',
        component: ComponentCreator('/tutorials/examples/guest-book', 'c73'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/hello-near',
        component: ComponentCreator('/tutorials/examples/hello-near', 'aa9'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/examples/xcc',
        component: ComponentCreator('/tutorials/examples/xcc', 'a8e'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/fts/simple-fts',
        component: ComponentCreator('/tutorials/fts/simple-fts', '9c9'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/approvals',
        component: ComponentCreator('/tutorials/nfts/approvals', '0ef'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/core',
        component: ComponentCreator('/tutorials/nfts/core', '252'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/enumeration',
        component: ComponentCreator('/tutorials/nfts/enumeration', 'c69'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/events',
        component: ComponentCreator('/tutorials/nfts/events', 'e26'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/introduction',
        component: ComponentCreator('/tutorials/nfts/introduction', '43c'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/approvals',
        component: ComponentCreator('/tutorials/nfts/js/approvals', '3d3'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/core',
        component: ComponentCreator('/tutorials/nfts/js/core', '037'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/enumeration',
        component: ComponentCreator('/tutorials/nfts/js/enumeration', '505'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/events',
        component: ComponentCreator('/tutorials/nfts/js/events', '464'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/introduction',
        component: ComponentCreator('/tutorials/nfts/js/introduction', '153'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/marketplace',
        component: ComponentCreator('/tutorials/nfts/js/marketplace', 'ccd'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/minting',
        component: ComponentCreator('/tutorials/nfts/js/minting', '412'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/predeployed-contract',
        component: ComponentCreator('/tutorials/nfts/js/predeployed-contract', '25b'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/royalty',
        component: ComponentCreator('/tutorials/nfts/js/royalty', 'a32'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/skeleton',
        component: ComponentCreator('/tutorials/nfts/js/skeleton', '8ee'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/js/upgrade-contract',
        component: ComponentCreator('/tutorials/nfts/js/upgrade-contract', 'c80'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/marketplace',
        component: ComponentCreator('/tutorials/nfts/marketplace', '3ea'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/minecraft-nfts',
        component: ComponentCreator('/tutorials/nfts/minecraft-nfts', 'd5f'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/minting',
        component: ComponentCreator('/tutorials/nfts/minting', '26f'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/minting-nft-frontend',
        component: ComponentCreator('/tutorials/nfts/minting-nft-frontend', '743'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/minting-nfts',
        component: ComponentCreator('/tutorials/nfts/minting-nfts', '266'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/predeployed-contract',
        component: ComponentCreator('/tutorials/nfts/predeployed-contract', '247'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/royalty',
        component: ComponentCreator('/tutorials/nfts/royalty', '080'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/series',
        component: ComponentCreator('/tutorials/nfts/series', '35e'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/skeleton',
        component: ComponentCreator('/tutorials/nfts/skeleton', 'baa'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/nfts/upgrade-contract',
        component: ComponentCreator('/tutorials/nfts/upgrade-contract', 'bff'),
        exact: true,
        sidebar: "tutorials"
      },
      {
        path: '/tutorials/welcome',
        component: ComponentCreator('/tutorials/welcome', 'd84'),
        exact: true,
        sidebar: "tutorials"
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
