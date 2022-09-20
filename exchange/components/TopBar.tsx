import { useCallback, useState } from 'react'
import Link from 'next/link'
import { abbreviateAddress } from '../utils/index'
import useLocalStorageState from '../hooks/useLocalStorageState'
import MenuItem from './MenuItem'
import useMangoStore from '../stores/useMangoStore'
import { ConnectWalletButton } from 'components'
// import NavDropMenu from './NavDropMenu'
import AccountsModal from './AccountsModal'
import { DEFAULT_MARKET_KEY, initialMarket } from './SettingsModal'
import { useTranslation } from 'next-i18next'
import Settings from './Settings'
import TradeNavMenu from './TradeNavMenu'
// import {
//   CalculatorIcon,
//   CurrencyDollarIcon,
//   LibraryIcon,
//   LightBulbIcon,
//   UserAddIcon,
// } from '@heroicons/react/outline'
// import { MangoIcon, TrophyIcon } from './icons'
import { useWallet } from '@solana/wallet-adapter-react'

// const StyledNewLabel = ({ children, ...props }) => (
//   <div style={{ fontSize: '0.5rem', marginLeft: '1px' }} {...props}>
//     {children}
//   </div>
// )

const TopBar = () => {
  const { t } = useTranslation('common')
  const { connected, publicKey } = useWallet()
  const mangoAccount = useMangoStore((s) => s.selectedMangoAccount.current)
  const mangoAccounts = useMangoStore((s) => s.mangoAccounts)
  const cluster = useMangoStore((s) => s.connection.cluster)
  const [showAccountsModal, setShowAccountsModal] = useState(false)
  const [defaultMarket] = useLocalStorageState(
    DEFAULT_MARKET_KEY,
    initialMarket
  )
  const isDevnet = cluster === 'devnet'

  const handleCloseAccounts = useCallback(() => {
    setShowAccountsModal(false)
  }, [])

  return (
    <>
      <nav className={`bg-th-bkg-2`}>
        <div className={`px-4 xl:px-6`}>
          <div className={`flex h-14 justify-between`}>
            <div className={`flex`}>
              <Link href={defaultMarket.path} shallow={true}>
                <div
                  className={`flex flex-shrink-0 cursor-pointer items-center`}
                >
                  <img
                    className={`h-8 w-auto`}
                    src="/assets/icons/logo.png"
                    alt="next"
                  />
                </div>
              </Link>
              <div
                className={`hidden md:ml-4 md:flex md:items-center md:space-x-2 lg:space-x-3`}
              >
                <MenuItem href="/markets">{t('markets')}</MenuItem>
                <TradeNavMenu />
                <MenuItem href="/account">{t('account')}</MenuItem>
                {/* <MenuItem href="/governance">{t('Governance')}</MenuItem> */}
                {/* <MenuItem href="/swap">{t('swap')}</MenuItem> */}
                {/* <div className="relative">
                  <MenuItem href="/leaderboard">
                    {t('leaderboard')}
                    <div className="absolute -right-3 -top-3 flex h-4 items-center justify-center rounded-full bg-gradient-to-br from-red-500 to-yellow-500 px-1.5">
                      <StyledNewLabel className="uppercase text-white">
                        new
                      </StyledNewLabel>
                    </div>
                  </MenuItem>
                </div> */}
                {/* <MenuItem href="/stats">{t('stats')}</MenuItem> */}
                {/* <NavDropMenu
                  menuTitle={t('more')}
                  // linksArray: [name: string, href: string, isExternal: boolean]
                  linksArray={[
                    [
                      t('Coming Soon'),
                      '/coming-soon',
                      false,
                      <UserAddIcon className="h-4 w-4" key="referrals" />,
                    ],
                    // [
                    //   t('leaderboard'),
                    //   '/leaderboard',
                    //   false,
                    //   <TrophyIcon className="h-4 w-4" key="leaderboard" />,
                    // ],
                    // [
                    //   t('calculator'),
                    //   '/risk-calculator',
                    //   false,
                    //   <CalculatorIcon className="h-4 w-4" key="calculator" />,
                    // ],
                    // [
                    //   t('fees'),
                    //   '/fees',
                    //   false,
                    //   <CurrencyDollarIcon className="h-4 w-4" key="fees" />,
                    // ],
                    // [
                    //   t('learn'),
                    //   'https://docs.mango.markets/',
                    //   true,
                    //   <LightBulbIcon className="h-4 w-4" key="learn" />,
                    // ],
                    // [
                    //   t('governance'),
                    //   'https://dao.mango.markets/',
                    //   true,
                    //   <LibraryIcon className="h-4 w-4" key="governance" />,
                    // ],
                    // [
                    //   'Mango v2',
                    //   'https://v2.mango.markets',
                    //   true,
                    //   <MangoIcon
                    //     className="h-4 w-4 stroke-current"
                    //     key="mango-v2"
                    //   />,
                    // ],
                    // [
                    //   'Mango v1',
                    //   'https://v1.mango.markets',
                    //   true,
                    //   <MangoIcon
                    //     className="h-4 w-4 stroke-current"
                    //     key="mango-v1"
                    //   />,
                    // ],
                  ]}
                /> */}
              </div>
            </div>
            <div className="flex items-center space-x-2.5">
              {isDevnet ? <div className="pl-2 text-xxs">Devnet</div> : null}
              <div className="pl-2">
                <Settings />
              </div>
              {mangoAccount &&
              mangoAccount.owner.toBase58() === publicKey?.toBase58() ? (
                <button
                  className="rounded border border-th-bkg-4 py-1 px-2 text-xs focus:outline-none md:hover:border-th-fgd-4"
                  onClick={() => setShowAccountsModal(true)}
                >
                  <div className="text-xs font-normal text-th-primary">
                    {mangoAccounts
                      ? mangoAccounts.length === 1
                        ? `1 ${t('account')}`
                        : `${mangoAccounts.length} ${t('accounts')}`
                      : t('account')}
                  </div>
                  {mangoAccount.name
                    ? mangoAccount.name
                    : abbreviateAddress(mangoAccount.publicKey)}
                </button>
              ) : connected && !mangoAccount ? (
                <button
                  className="rounded border border-th-bkg-4 py-1 px-2 text-xs focus:outline-none md:hover:border-th-fgd-4"
                  onClick={() => setShowAccountsModal(true)}
                >
                  <div className="text-xs font-normal text-th-primary">
                    {`0 ${t('accounts')}`}
                  </div>
                  {t('get-started')} 😎
                </button>
              ) : null}
              <ConnectWalletButton />
            </div>
          </div>
        </div>
      </nav>
      {showAccountsModal && (
        <AccountsModal
          onClose={handleCloseAccounts}
          isOpen={showAccountsModal}
        />
      )}
    </>
  )
}

export default TopBar
