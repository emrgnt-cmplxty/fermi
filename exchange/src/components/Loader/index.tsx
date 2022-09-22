import { ReactElement } from 'react'

function Loader(): ReactElement {
  return (
    <div
      style={{
        display: 'flex',
        height: '100%',
        width: '100%',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      Loader loading...
    </div>
  )
}

export default Loader
