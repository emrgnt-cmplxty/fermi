import 'simplebar/src/simplebar.css'
// TODO: consider removing from cssbaseline?
import './styles/_global.scss'

import { Web3ReactProvider } from '@web3-react/core'
import { providers as ethersProviders } from 'ethers'
import { SnackbarProvider } from 'notistack'
import CurrentMarketProvider from 'providers/CurrentMarketProvider'
import CurrentOrderProvider from 'providers/CurrentOrderProvider'
import { LanguageProvider } from 'providers/LanguageProvider'
import QueryProvider from 'providers/QueryProvider'
import SettingsProvider from 'providers/SettingsProvider'
import ThemeProvider from 'providers/ThemeProvider'
import { Web3ContextProvider } from 'providers/Web3Provider'
import React from 'react'
import { createRoot } from 'react-dom/client'
import { ReactQueryDevtools } from 'react-query/devtools'
import { BrowserRouter } from 'react-router-dom'

import App from './App'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function getWeb3Library(provider: any): ethersProviders.Web3Provider {
  const library = new ethersProviders.Web3Provider(provider)
  library.pollingInterval = 12000
  return library
}
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const container = document.getElementById('root')!
const root = createRoot(container)

root.render(
  <React.StrictMode>
    <LanguageProvider>
      <Web3ReactProvider getLibrary={getWeb3Library}>
        <Web3ContextProvider>
          <CurrentOrderProvider>
            <CurrentMarketProvider>
              <SettingsProvider>
                <BrowserRouter>
                  <ThemeProvider>
                    <SnackbarProvider maxSnack={3}>
                      <QueryProvider>
                        <App />
                        <ReactQueryDevtools initialIsOpen={false} />
                      </QueryProvider>
                    </SnackbarProvider>
                  </ThemeProvider>
                </BrowserRouter>
              </SettingsProvider>
            </CurrentMarketProvider>
          </CurrentOrderProvider>
        </Web3ContextProvider>
      </Web3ReactProvider>
    </LanguageProvider>
  </React.StrictMode>,
)
