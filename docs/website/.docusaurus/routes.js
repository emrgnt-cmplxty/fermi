import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '186'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', 'dfb'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', 'e5d'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', '983'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '41f'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', 'f0b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', 'f80'),
    exact: true
  },
  {
    path: '/search',
    component: ComponentCreator('/search', '581'),
    exact: true
  },
  {
    path: '/tools/near-api-js/reference',
    component: ComponentCreator('/tools/near-api-js/reference', '3d5'),
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
    component: ComponentCreator('/tools/near-sdk-js/reference', '5d7'),
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
    component: ComponentCreator('/', 'f87'),
    routes: [
      {
        path: '/',
        component: ComponentCreator('/', '5cd'),
        exact: true
      },
      {
        path: '/api/rpc/access-keys',
        component: ComponentCreator('/api/rpc/access-keys', '094'),
        exact: true
      },
      {
        path: '/api/rpc/block-info',
        component: ComponentCreator('/api/rpc/block-info', '33d'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/contracts',
        component: ComponentCreator('/api/rpc/contracts', 'f7f'),
        exact: true
      },
      {
        path: '/api/rpc/futures-markets',
        component: ComponentCreator('/api/rpc/futures-markets', '0c4'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/gas',
        component: ComponentCreator('/api/rpc/gas', '367'),
        exact: true
      },
      {
        path: '/api/rpc/introduction',
        component: ComponentCreator('/api/rpc/introduction', '7c8'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/network',
        component: ComponentCreator('/api/rpc/network', '5a4'),
        exact: true
      },
      {
        path: '/api/rpc/protocol',
        component: ComponentCreator('/api/rpc/protocol', '39f'),
        exact: true
      },
      {
        path: '/api/rpc/providers',
        component: ComponentCreator('/api/rpc/providers', 'ccf'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/setup',
        component: ComponentCreator('/api/rpc/setup', 'a60'),
        exact: true,
        sidebar: "api"
      },
      {
        path: '/api/rpc/transactions',
        component: ComponentCreator('/api/rpc/transactions', 'd11'),
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
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
