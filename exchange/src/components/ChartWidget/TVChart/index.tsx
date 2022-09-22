import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useEffect } from 'react'

import {
  ChartingLibraryWidgetOptions,
  ResolutionString,
  widget,
} from '../../../public/charting_library/charting_library.esm.js'

interface TVChartProps {
  height: number
  width: number
  symbol: string
}
// TODO, remove SYMBOl hack later
const TVChart = ({ height, width, symbol }: TVChartProps) => {
  const { settingsState } = useContext(SettingsContext)
  useEffect(() => {
    const baseUrl = 'https://event-history-api-candles.herokuapp.com'

    const CHART_DATA_FEED = `${baseUrl}/tv`

    const mainSeriesProperties = [
      'candleStyle',
      'hollowCandleStyle',
      'haStyle',
      'barStyle',
    ]

    let chartStyleOverrides = {}
    mainSeriesProperties.forEach((prop) => {
      chartStyleOverrides = {
        ...chartStyleOverrides,
        // [`mainSeriesProperties.${prop}.barColorsOnPrevClose`]: true,
        // [`mainSeriesProperties.${prop}.drawWick`]: true,
        // [`mainSeriesProperties.${prop}.drawBorder`]: true,
        // [`mainSeriesProperties.${prop}.upColor`]: '#5EBF4D',
        // [`mainSeriesProperties.${prop}.downColor`]: '#CC2929',
        // [`mainSeriesProperties.${prop}.borderColor`]: '#5EBF4D',
        // [`mainSeriesProperties.${prop}.borderUpColor`]: '#5EBF4D',
        // [`mainSeriesProperties.${prop}.borderDownColor`]: '#CC2929',
        // [`mainSeriesProperties.${prop}.wickUpColor`]: '#5EBF4D',
        // [`mainSeriesProperties.${prop}.wickDownColor`]: '#CC2929',
      }
    })

    const defaultProps: any = {
      symbol: 'BTC-PERP', // symbol.replace('USD', 'PERP'),
      interval: '60' as ResolutionString,
      theme: 'Dark',
      containerId: 'tv_chart_container',
      datafeedUrl: CHART_DATA_FEED,
      libraryPath: '/charting_library/',
      // fullscreen: true,
      //autosize: true,
      studiesOverrides: {
        'volume.volume.color.0': '#CC2929',
        'volume.volume.color.1': '#5EBF4D',
        'volume.precision': 4,
      },
    }

    const widgetOptions: ChartingLibraryWidgetOptions = {
      // debug: true,
      symbol: defaultProps.symbol,
      // BEWARE: no trailing slash is expected in feed URL
      // tslint:disable-next-line:no-any
      datafeed: new (window as any).Datafeeds.UDFCompatibleDatafeed(
        defaultProps.datafeedUrl,
      ),
      interval:
        defaultProps.interval as ChartingLibraryWidgetOptions['interval'],
      container_id:
        defaultProps.containerId as ChartingLibraryWidgetOptions['container_id'],
      library_path: defaultProps.libraryPath as string,
      height: height - 50,
      width: width,
      locale: 'en',
      disabled_features: [
        //   'header_undo_redo',
        'volume_force_overlay',
        'use_localstorage_for_settings',
      ],
      enabled_features: ['hide_left_toolbar_by_default'],
      // enabled_features: ['enable_publishing', 'control_bar'],
      // disabled_features: [
      //   'use_localstorage_for_settings',
      //   'timeframes_toolbar',
      //   'volume_force_overlay',
      //   //'left_toolbar',
      //   'show_logo_on_all_charts',
      //   'caption_buttons_text_if_possible',
      //   'header_settings',
      //   // 'header_chart_type',
      //   'header_compare',
      //   'compare_symbol',
      //   'header_screenshot',
      //   //'header_widget_dom_node',
      //   //'header_widget',
      //   'header_saveload',
      //   'header_undo_redo',
      //   'header_interval_dialog_button',
      //   'show_interval_dialog_on_key_press',
      //   'header_symbol_search',
      // ],
      load_last_chart: true,
      client_id: defaultProps.clientId,
      user_id: defaultProps.userId,
      fullscreen: defaultProps.fullscreen,
      autosize: defaultProps.autosize,
      studies_overrides: defaultProps.studiesOverrides,
      theme: settingsState.theme === 'LIGHT' ? 'Light' : 'Dark',
      //custom_css_url: '/tradingview-chart.css',
      loading_screen: {
        backgroundColor:
          settingsState.theme === 'LIGHT' ? '#ffffff' : '#1B1B1F',
      },
      overrides: {
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        'paneProperties.background':
          settingsState.theme === 'LIGHT' ? '#ffffff' : '#101820',
        ...chartStyleOverrides,
      },
    }
    const tvWidget = new widget(widgetOptions)
    tvWidget?._innerWindowLoaded &&
      tvWidget.onChartReady(() => {
        tvWidget.headerReady().then(() => {
          const button = tvWidget.createButton()
          button.setAttribute('title', 'Click to show a notification popup')
          button.classList.add('apply-common-tooltip')
          /*button.addEventListener('click', () =>
            tvWidget.showNoticeDialog({
                title: 'Notification',
                body: 'TradingView Charting Library API works correctly',
                callback: () => {
                console.log('Noticed!')
                },
            }),
            )*/

          button.innerHTML = 'Check API'
        })
      })
  }, [settingsState.theme, symbol, height])
  return <div id="tv_chart_container" key={settingsState.theme} />
}

export default TVChart
