import { Grid, Typography } from '@mui/material'

// TODO
// 1.) Why is price not aligned with size and total?
interface TitleRowProps {
  baseSymbol: string
  quoteSymbol: string
}

const TitleRow = ({ baseSymbol, quoteSymbol }: TitleRowProps) => {
  return (
    <Grid container direction="row">
      <Grid item xs={4} justifyContent="flex-start" sx={{ mt: -0.3, mb: 0.3 }}>
        <Typography variant="caption" color="textSecondary" alignItems="center">
          {`Price (${quoteSymbol})`}
        </Typography>
      </Grid>
      <Grid item xs={4} container justifyContent="flex-end" alignItems="center">
        <Typography variant="caption" color="textSecondary">
          {`Size (${baseSymbol})`}
        </Typography>
      </Grid>
      <Grid item xs={4} container justifyContent="flex-end" alignItems="center">
        <Typography variant="caption" color="textSecondary">
          {`Total (${baseSymbol})`}
        </Typography>
      </Grid>
    </Grid>
  )
}

export default TitleRow
