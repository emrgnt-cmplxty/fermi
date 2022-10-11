import { ReactElement } from 'react'
import { Outlet } from 'react-router-dom'

function PageLayout(): ReactElement {
  return (
    <>
      <Outlet />
    </>
  )
}

export default PageLayout
