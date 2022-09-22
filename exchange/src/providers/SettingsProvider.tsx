import {
  createContext,
  Dispatch,
  ReactNode,
  useContext,
  useEffect,
  useReducer,
  useState,
} from 'react'
import { ThemeKey } from 'theme/theme'
import { MarketSymbol } from 'utils/globals'
import { CURRENCIES, LANGUAGES } from 'utils/settingsConstants'

//note: if any of these settings rely on react-query context, they should be moved to an inner context
export type Settings = {
  compact?: boolean
  direction?: 'ltr' | 'rtl'
  availableMyAlgoWallets?: string[]
  // responsiveFontSizes?: boolean
  theme?: ThemeKey
  language?: string
  currency?: string
  favorites?: MarketSymbol[]
  shortMaxLeverage?: string
}

type SettingsAction = {
  type: 'updateSetting'
  payload: Partial<Settings>
}
// | {
//     type: 'setSettings'
//     payload: Settings
//   }

export interface SettingsContextInterface {
  settingsState: Settings
  settingsDispatch: Dispatch<SettingsAction>
  isLoading: boolean
}

const initialState: Settings = {
  compact: true,
  direction: 'ltr',
  // responsiveFontSizes: true,
  theme: 'DARK',
  language: LANGUAGES.ENGLISH,
  currency: CURRENCIES.USD,
  favorites: [],
}

export const getStoredSettings = (): Settings => {
  const settings = null
  const storedData: string | null =
    window.localStorage.getItem('algofiSettings')
  return storedData ? JSON.parse(storedData) : settings
}

export const storeSettings = (settings: Settings): void => {
  window.localStorage.setItem('algofiSettings', JSON.stringify(settings))
}

export const SettingsContext = createContext<SettingsContextInterface>({
  settingsState: initialState,
  settingsDispatch: () => {},
  isLoading: true,
})

export const useSetting = (key: keyof Settings) => {
  const { settingsState } = useContext(SettingsContext)
  return settingsState[key]
}

interface SettingsProviderProps {
  children: ReactNode
}

function SettingsProvider({ children }: SettingsProviderProps) {
  const [isLoading, setIsLoading] = useState(true)

  const settingsReducer = (
    state: Settings,
    action: SettingsAction,
  ): Settings => {
    switch (action.type) {
      case 'updateSetting': {
        const newSettings = Object.assign({}, state, action.payload)
        storeSettings(newSettings)
        return newSettings
      }
      default:
        return state
    }
  }

  const [settingsState, settingsDispatch] = useReducer(
    settingsReducer,
    initialState as Settings,
  )

  useEffect(() => {
    const restoreSettings = async () => {
      setIsLoading(true)
      const storedSettings = getStoredSettings()

      if (storedSettings) {
        settingsDispatch({
          type: 'updateSetting',
          payload: storedSettings,
        })
      }
      setIsLoading(false)
    }
    restoreSettings()
  }, [])

  return (
    <SettingsContext.Provider
      value={{
        settingsState,
        settingsDispatch,
        isLoading,
      }}
    >
      {children}
    </SettingsContext.Provider>
  )
}

export default SettingsProvider
