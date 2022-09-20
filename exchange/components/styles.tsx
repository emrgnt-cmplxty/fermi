export const PageBodyWrapper = ({ children, className, ...props }) => (
  <div className={`${className} h-full`} {...props}>
    {children}
  </div>
)

export const ElementTitle = ({
  children,
  noMarginBottom = false,
  className = '',
}) => (
  <h6
    className={`flex text-m justify-center ${
      noMarginBottom ? 'mb-0' : 'mb-2.5'
    } -mt-2 mb-1 -ml-1 items-center text-base ${className}`}
  >
    {children}
  </h6>
)
