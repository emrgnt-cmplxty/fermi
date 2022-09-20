import {
  Box,
  Button,
  Fade,
  Grid,
  Modal,
  Slider,
  TextField,
  Typography,
} from '@mui/material'
import { useSelectMarketDataSymbol } from 'hooks/react-query/useMarketOverview'
import { useUserData } from 'hooks/react-query/useUser'
import { useState } from 'react'
import { MarketSymbol } from 'utils/globals'

import styles from './index.module.scss'

interface CloseInfoProps {
  marketSymbol: MarketSymbol
  slide: number
  closeType: 'MARKET' | 'LIMIT'
  closePrice: number
}
const CloseInfo = ({
  marketSymbol,
  slide,
  closeType,
  closePrice,
}: CloseInfoProps) => {
  const { data: userData } = useUserData({ refetchInterval: 1000 })
  const marketData = userData?.positions.find((ele) => {
    return ele.marketSymbol === marketSymbol
  })
  return (
    <Box style={{ width: 1000 }}>
      <Typography variant="caption" sx={{ mt: 1, ml: 1, mr: 1 }}>
        {`${(slide / 100) * marketData?.size} contract(s) will be closed at ${
          closeType === 'MARKET'
            ? 'Last Traded Price'
            : closePrice.toString() + ' USD'
        }.`}
      </Typography>
    </Box>
  )
}

interface CloseSliderProps {
  marketSymbol: MarketSymbol
  slide: number
  setSlider: Dispatch<SetStateAction<number>>
}
const CloseSlider = ({ marketSymbol, slide, setSlider }: CloseSliderProps) => {
  const { data: userData } = useUserData({ refetchInterval: 1000 })
  const marketData = userData?.positions.find((ele) => {
    return ele.marketSymbol === marketSymbol
  })
  const [displayVal, setDisplayVal] = useState(
    ((slide / 100) * marketData?.size).toString(),
  )
  return (
    <>
      {/* TODO - setup pipes for short slider  */}
      <Typography>Amount to Close</Typography>
      <TextField
        id="outlined-name"
        type={'string'}
        value={displayVal}
        inputProps={{ min: 0, style: { textAlign: 'center' } }}
        onChange={(event) => {
          if (
            event?.target.value >= 0 &&
            event?.target.value <= marketData?.size
          ) {
            setDisplayVal(event?.target.value)
            setSlider((event?.target.value / marketData?.size) * 100)
          }
        }}
      />
      <Slider
        sx={{
          '& input[type="range"]': {
            WebkitAppearance: 'slider-vertical',
          },
        }}
        value={slide}
        max={100}
        step={2.5}
        onChange={(event) => {
          setSlider(event?.target.value)
          setDisplayVal((event?.target.value * marketData?.size) / 100)
        }}
      />
    </>
  )
}

interface CloseQuantityProps {
  closePrice: number
  setClosePrice: React.Dispatch<React.SetStateAction<number>>
}
const CloseQuantity = ({ closePrice, setClosePrice }: CloseQuantityProps) => {
  return (
    <>
      {/* TODO - setup pipes for short slider  */}
      <Typography>Close Price USD</Typography>
      <TextField
        id="outlined-name"
        type={'string'}
        value={closePrice}
        inputProps={{ min: 0, style: { textAlign: 'center' } }}
        onChange={(event) => {
          setClosePrice(event?.target.value)
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

interface CloseBodyProps {
  setOpenModal: Dispatch<SetStateAction<boolean>>
  marketSymbol: MarketSymbol
  closeType: 'MARKET' | 'LIMIT'
}

const CloseBody = ({
  setOpenModal,
  marketSymbol,
  closeType,
}: CloseBodyProps) => {
  const [slide, setSlider] = useState(100)
  const { data: selectedMarket } = useSelectMarketDataSymbol(marketSymbol, {
    refetchInterval: 1000,
  })
  const [closePrice, setClosePrice] = useState(selectedMarket?.price)

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
      <Box sx={{ pb: 2 }}>{`${
        closeType === 'MARKET' ? 'Market' : 'Limit'
      } Close ${marketSymbol}`}</Box>
      <CloseSlider
        marketSymbol={marketSymbol}
        slide={slide}
        setSlider={setSlider}
      />
      <Box sx={{ pb: 1 }} />
      {closeType === 'LIMIT' ? (
        <CloseQuantity closePrice={closePrice} setClosePrice={setClosePrice} />
      ) : (
        ''
      )}
      <Box sx={{ pb: 2 }} />
      <CloseInfo
        marketSymbol={marketSymbol}
        slide={slide}
        closeType={closeType}
        closePrice={closePrice}
      />
      <Box sx={{ pb: 2 }} />

      {/* TODO -- make leverageErr globally defined */}
      {/* {leverageErr && <Typography variant="h6" color="red" sx={{ml:1, mt:2, mb: -2}}> The specified leverage is to high, please select a lower value</Typography>} */}
      <CancelOrConfirm leverageErr={leverageErr} setOpenModal={setOpenModal} />
    </Box>
  )
}

interface PositionCloseModalProps {
  closeType: 'MARKET' | 'LIMIT'
  openModal: boolean
  setOpenModal: Dispatch<SetStateAction<boolean>>
  marketSymbol: MarketSymbol
}

export default function PositionCloseModal({
  closeType,
  openModal,
  setOpenModal,
  marketSymbol,
}: PositionCloseModalProps) {
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
      <Fade>
        {openModal && (
          <CloseBody
            setOpenModal={setOpenModal}
            marketSymbol={marketSymbol}
            closeType={closeType}
          />
        )}
      </Fade>
    </Modal>
  )
}
