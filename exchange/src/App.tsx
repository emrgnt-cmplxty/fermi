import 'styles/react-grid.css'

import NavigationScroll from 'layouts/NavigationScroll'
import { ReactElement } from 'react'
import AppRoutes from 'routes/routes'

function App(): ReactElement {
  return (
    <>
      <NavigationScroll />

      <AppRoutes />
    </>
  )
}

export default App
