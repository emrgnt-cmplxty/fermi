import { Grid, List, ListItemButton } from '@mui/material'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext } from 'react'
import { CURRENCIES, LANGUAGES } from 'utils/settingsConstants'

const LanguageCCYSettings = () => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)

  const handleModifyLanguage = (language: string) => {
    settingsDispatch({
      type: 'updateSetting',
      payload: {
        language: language,
      },
    })
  }

  const handleModifyCurrency = (currency: string) => {
    settingsDispatch({
      type: 'updateSetting',
      payload: {
        currency: currency,
      },
    })
  }
  return (
    <Grid container direction="row">
      <Grid item xs={6} sx={{ maxHeight: 250 }}>
        <List>
          {Object.entries(LANGUAGES).map(([key, value]) => {
            return (
              <ListItemButton
                key={key}
                sx={{ width: 150 }}
                selected={value === settingsState?.language}
                onClick={() => {
                  handleModifyLanguage(value)
                }}
              >
                {value}
              </ListItemButton>
            )
          })}
        </List>
      </Grid>
      <Grid item xs={6}>
        <List>
          {Object.entries(CURRENCIES).map(([key, value]) => {
            return (
              <ListItemButton
                key={key}
                sx={{ width: 150 }}
                selected={value === settingsState?.currency}
                onClick={() => {
                  handleModifyCurrency(value)
                }}
              >
                {value}
              </ListItemButton>
            )
          })}
        </List>
      </Grid>
    </Grid>
  )
}
export default LanguageCCYSettings
