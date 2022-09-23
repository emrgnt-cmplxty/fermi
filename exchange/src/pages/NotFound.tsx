import { ReactElement } from 'react'
import { Link } from 'react-router-dom'

function NotFound(): ReactElement {
  return (
    <div
      style={{
        display: 'flex',
        minHeight: '100%',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <div style={{ textAlign: 'center', maxWidth: 480 }}>
        <h3>Sorry, page not found!</h3>
        <p>
          Sorry, we couldn’t find the page you’re looking for. Perhaps you’ve
          mistyped the URL? Be sure to check your spelling.
        </p>

        <Link to="/">
          <button> Go to Home</button>
        </Link>
      </div>
    </div>
  )
}

export default NotFound
