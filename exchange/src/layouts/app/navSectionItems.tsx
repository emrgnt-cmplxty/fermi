export interface NavItem {
  title: string
  path: string
  icon?: JSX.Element
  info?: string
  children?: NavItem[]
  subheader?: string
}

const navSectionItems: NavItem[] = [
  {
    title: 'markets',
    path: '/app/markets',
  },
  // {
  //   title: 'spot',
  //   path: '/app/trade/BTC-USD',
  // },
  {
    title: 'futures',
    path: '/app/futures/BTC-PERP',
    children: [
      {
        title: 'Light Mode',
        subheader: 'Futures trading made easy',
        path: '/app/futures-light/BTC-PERP',
      },
      {
        title: '3x Tokens',
        subheader: 'Leverage without loans or margin',
        path: '/app/leveraged-tokens/BTC',
      },
      {
        title: 'Pro Mode',
        subheader: 'Comprehensive trading with more features',
        path: '/app/futures/BTC-PERP',
      },
    ],
  },
  // { title: 'portfolio', path: '/app/portfolio/dashboard' },
  {
    title: 'vaults',
    path: '/app/vaults',
    children: [
      { title: 'x1', path: '/app/trade/x1' },
      { title: 'x2', path: '/app/trade/quarterly' },
      { title: 'x3', path: '/app/trade/volatility' },
      { title: 'x4', path: '/app/trade/funding' },
    ],
  },
  {
    title: 'governance',
    path: '/app/governance',
  },
  {
    title: 'faucet',
    path: '/app/faucet',
  },
  {
    title: 'more',
    path: '/app/more',
    children: [
      { title: 'x1', path: '/app/trade/x1' },
      { title: 'x2', path: '/app/trade/quarterly' },
      { title: 'x3', path: '/app/trade/volatility' },
      { title: 'x4', path: '/app/trade/funding' },
    ],
  },
]

export default navSectionItems
