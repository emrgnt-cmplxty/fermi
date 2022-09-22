import CloseIcon from '@mui/icons-material/Close'
import {
  Button,
  Card,
  Divider,
  Grid,
  IconButton,
  Tab,
  Tabs,
  Tooltip,
  Typography,
} from '@mui/material'

const WidgetCloseIcon = () => {
  return (
    <Tooltip title="Close" sx={{ width: 0, mt: -1 }}>
      <IconButton
        aria-label="delete"
        // onClick={() => onRemoveItem(id)}
      >
        <CloseIcon style={{ fontSize: 12 }} />
      </IconButton>
    </Tooltip>
  )
}
export default WidgetCloseIcon
