import {
  Box,
  Button,
  Fade,
  Grid,
  Modal,
  Radio,
  Slider,
  TextField,
  Typography,
} from '@mui/material'
import { useWeb3Context } from 'hooks/useWeb3Context'
import { useState } from 'react'

import styles from './index.module.scss'

const marks = [
  {
    value: 1,
    label: '1x',
  },
  {
    value: 3,
    label: '3x',
  },
  {
    value: 5,
    label: '5x',
  },
  {
    value: 10,
    label: '10x',
  },
  {
    value: 20,
    label: '20x',
  },
]

const ModeSelector = () => {
  const [isIsolated, setIsIsolated] = useState(true)

  return (
    <>
      <Grid container direction="row" spacing={2}>
        <Grid item xs={6}>
          <Button
            fullWidth
            variant={isIsolated ? 'contained' : 'outlined'}
            sx={{ height: 40 }}
            onClick={() => {
              setIsIsolated(true)
            }}
          >
            <Radio
              color={isIsolated ? 'buttonText' : 'primary'}
              checked={isIsolated}
              sx={{ ml: -2 }}
              size="small"
            />
            <Typography
              sx={{ color: isIsolated ? 'buttonText.main' : 'primary' }}
              variant="h6"
            >
              {' '}
              Isolated{' '}
            </Typography>
          </Button>
        </Grid>
        <Grid item xs={6}>
          <Button
            fullWidth
            variant={!isIsolated ? 'contained' : 'outlined'}
            sx={{ height: 40 }}
            onClick={() => {
              setIsIsolated(false)
            }}
          >
            <Radio
              color={!isIsolated ? 'buttonText' : 'primary'}
              checked={!isIsolated}
              sx={{ ml: -2 }}
              size="small"
            />
            <Typography
              sx={{ color: !isIsolated ? 'buttonText.main' : 'primary' }}
              variant="h6"
            >
              {' '}
              Cross{' '}
            </Typography>
          </Button>
        </Grid>
      </Grid>
      {!isIsolated && (
        <Typography variant="caption" sx={{ mt: 1, ml: 1, mr: 1 }}>
          Under cross margin, all available balance of the corresponding margin
          account will be deployed to meet maintenance margin requirements and
          prevent liquidation. All corresponding available balance can be lost
          in the event of liquidation. Please note that adjusting the leverage
          will affect all positions and active orders under the current pair.
        </Typography>
      )}
      {isIsolated && (
        <Typography variant="caption" sx={{ mt: 1, ml: 1, mr: 1 }}>
          Under isolated margin, a specific amount of margin, i.e. initial
          margin, is applied to a position, and position margin can be adjusted
          manually. In the event of a liquidation, you may lose the initial
          margin and extra margin added to this position. Please note that
          adjusting the leverage will affect all positions and active orders
          under the current pair.
        </Typography>
      )}
    </>
  )
}

const LongSlider = () => {
  const [longLev, setLongLev] = useState(20)
  return (
    <>
      {/* TODO - setup pipes for long slider  */}
      <Typography>Leverage Long</Typography>
      <TextField
        id="outlined-name"
        //label="Name"
        value={longLev}
        //onChange={handleChange}
        inputProps={{ min: 0, style: { textAlign: 'center' } }}
        onChange={(event) => {
          setLongLev(event?.target.value)
        }}
      />
      <Slider
        sx={{
          '& input[type="range"]': {
            WebkitAppearance: 'slider-vertical',
          },
        }}
        // defaultValue={longLev}
        value={longLev}
        aria-label="Temperature"
        valueLabelDisplay="auto"
        marks={marks}
        max={20}
        step={0.5}
        onChange={(event) => {
          setLongLev(event?.target.value)
        }}
      />
    </>
  )
}

const ShortSlider = () => {
  const [shortLev, setShortLev] = useState(20)

  return (
    <>
      {/* TODO - setup pipes for short slider  */}
      <Typography>Leverage Short</Typography>
      <TextField
        id="outlined-name"
        value={shortLev}
        inputProps={{ min: 0, style: { textAlign: 'center' } }}
        onChange={(event) => {
          setShortLev(event?.target.value)
        }}
      />
      <Slider
        sx={{
          '& input[type="range"]': {
            WebkitAppearance: 'slider-vertical',
          },
        }}
        // defaultValue={shortLev}
        value={shortLev}
        aria-label="Temperature"
        valueLabelDisplay="auto"
        marks={marks}
        max={20}
        step={0.5}
        onChange={(event) => {
          setShortLev(event?.target.value)
        }}
      />
    </>
  )
}

interface CancelOrConfirmProps {
  leverageErr: boolean
  setOpenModal: (newState: boolean) => void
}

const CancelOrConfirm = ({
  leverageErr,
  setOpenModal,
}: CancelOrConfirmProps) => {
  return (
    <>
      <Grid container direction="row" spacing={1}>
        <Grid item xs={6}>
          <Button
            fullWidth
            variant={'contained'}
            sx={{ height: 40 }}
            disabled={leverageErr}
            onClick={() => {
              setOpenModal(false)
            }}
          >
            <Typography sx={{ color: 'buttonText.main' }} variant="h6">
              {' '}
              Confirm{' '}
            </Typography>
          </Button>
        </Grid>
        <Grid item xs={6}>
          <Button
            fullWidth
            variant={'outlined'}
            sx={{ height: 40 }}
            onClick={() => {
              setOpenModal(false)
            }}
          >
            <Typography sx={{ color: 'primary' }} variant="h6">
              {' '}
              Cancel{' '}
            </Typography>
          </Button>
        </Grid>
      </Grid>
    </>
  )
}

interface LeverageBodyProps {
  setOpenModal: Dispatch<SetStateAction<boolean>>
}

const LeverageBody = ({ setOpenModal }: LeverageBodyProps) => {
  //const leverageErr = longLev > 20 || shortLev > 20
  const leverageErr = false
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: 'background.navbar',
        maxWidth: 425,
        height: 600,
        p: 2,
      }}
    >
      <Box sx={{ pb: 1 }}>Margin Settings</Box>
      <ModeSelector />
      <Box sx={{ mt: 2 }} />
      <LongSlider />
      <Box sx={{ mt: 2 }} />
      <ShortSlider />
      {/* TODO -- make leverageErr globally defined */}
      {leverageErr && (
        <Typography variant="h6" color="red" sx={{ ml: 1, mt: 2, mb: -2 }}>
          {' '}
          The specified leverage is to high, please select a lower value
        </Typography>
      )}
      <Box sx={{ mt: 3 }} />
      <CancelOrConfirm leverageErr={leverageErr} setOpenModal={setOpenModal} />
    </Box>
  )
}

interface LeverageModalProps {
  openModal: boolean
  setOpenModal: Dispatch<SetStateAction<boolean>>
}

export default function LeverageModal({
  openModal,
  setOpenModal,
}: LeverageModalProps) {
  const { publicAddress } = useWeb3Context()

  return (
    <Modal
      aria-labelledby="transition-modal-title"
      aria-describedby="transition-modal-description"
      open={openModal}
      onClose={(event, reason) => {
        setOpenModal(false)
      }}
      className={styles.modal}
      closeAfterTransition
    >
      <Fade>{openModal && <LeverageBody setOpenModal={setOpenModal} />}</Fade>
    </Modal>
  )
}
