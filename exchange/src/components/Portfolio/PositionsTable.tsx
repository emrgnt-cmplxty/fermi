import EditIcon from '@mui/icons-material/Edit'
import {
  Box,
  Button,
  Card,
  Grid,
  Skeleton,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
} from '@mui/material'
import { styled } from '@mui/system'
import { AssetDisplay } from 'components/AssetDisplay'
import CloseModal from 'components/PositionCloseModal'
import { StackedText } from 'components/StackedText/StackedText'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo, useState } from 'react'
import { Cell, Column, Row, useTable } from 'react-table'
import { Position } from 'utils/globals'
import { formatNumber } from 'utils/utils'

const StyledCard = styled(Card)(({ theme }) => ({
  backgroundColor: theme.palette.background.component,
}))

const StyledTableHead = styled(TableHead)(({ theme }) => ({
  backgroundColor: theme.palette.background.component,
}))

const StyledTableCell = styled(TableCell)(({ theme }) => ({
  backgroundColor: theme.palette.background.component,
}))

const type = 'futures'

const DUMMY_POSITIONS = [
  {
    marketSymbol: 'XRP-PERP',
    baseSymbol: 'XRP',
    baseName: 'Ripple',
    size: 10,
    notional: 10.1,
    entryPrice: 1.01,
    markPrice: 1.02,
    absMarginUsed: 4,
    percMarginUsed: 0.1,
    marginMode: 'cross', // cross or isolated
    pnlQuote: 1,
    pnlBase: 1,
    estLiqPrice: 0,
    leverage: 3.2,
  },
  {
    marketSymbol: 'BTC-PERP',
    baseSymbol: 'BTC',
    baseName: 'Bitcoin',
    size: 1,
    notional: 31034.1,
    entryPrice: 33123.1,
    markPrice: 31034.1,
    absMarginUsed: 0.2,
    percMarginUsed: 73,
    marginMode: 'cross', // cross or isolated
    pnlQuote: -3103,
    pnlBase: -0.1043,
    estLiqPrice: 0,
    leverage: 1.6,
  },
  {
    marketSymbol: 'ETH-PERP',
    baseSymbol: 'ETH',
    baseName: 'Ethereum',
    size: -100,
    notional: 200034.1,
    entryPrice: 1901.1,
    markPrice: 2000.1,
    absMarginUsed: 0.2,
    percMarginUsed: 73,
    marginMode: 'cross', // cross or isolated
    pnlQuote: -100000,
    pnlBase: -50,
    estLiqPrice: 3000,
    leverage: 0,
  },
]

// interface PositionsTableProps {}

export const PositionsTable: FunctionComponent<PositionsTableProps> = () => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)
  const [openClose, setOpenClose] = useState(false)

  const columns: Column<Position>[] = useMemo(
    () => [
      {
        Header: 'Contracts',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid>
            {
              <AssetDisplay
                symbol={value.marketSymbol}
                rightLabel={`${String(value.marketSymbol)}${
                  value.type === 'futures' ? '-PERP' : ''
                }`}
                rightMetaLabel={value.baseName}
                farRightLabel={
                  value.type === 'spot' ? `/${String(value.quoteSymbol)}` : ''
                }
              />
            }
            {/* {value.marketSymbol} */}
          </Grid>
        ),
      },
      {
        Header: 'Size',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid
            sx={{
              color: value.size > 0 ? 'tradeColors.bid' : 'tradeColors.ask',
            }}
          >
            {value.size}
          </Grid>
        ),
      },

      {
        Header: 'Leverage',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid>{`${value.leverage}x`}</Grid>
        ),
      },
      {
        Header: 'Liq. Price / Current',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid>
            <StackedText
              topText={`$${value.estLiqPrice}`}
              bottomText={`$${value.markPrice}`}
              textAlign="left"
            />
          </Grid>
        ),
      },
      {
        Header: 'Unrealized P&L',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid>
            {`${value.absMarginUsed} ${value.baseSymbol} (${value.percMarginUsed}%)`}
          </Grid>
        ),
      },
      // {
      //   Header: 'Margin Mode',
      //   accessor: (row) => {
      //     return row
      //   },
      //   Cell: ({ value }: { value: Position }) => (
      //     <Grid>
      //       {value.marginMode}
      //     </Grid>
      //   ),
      // },
      {
        Header: 'Realized P&L',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid
            sx={{
              color: value.pnlQuote > 0 ? 'tradeColors.bid' : 'tradeColors.ask',
            }}
          >
            {`${value.pnlQuote}`}
          </Grid>
        ),
      },

      {
        Header: 'Average Open / Close',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid>
            <StackedText
              topText={`$${value.entryPrice}`}
              bottomText={`--`}
              textAlign="left"
            />
          </Grid>
        ),
      },
      {
        Header: 'Reduce Position',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: Position }) => (
          <Grid
            container
            direction="row"
            sx={{ mb: -1 }}
            justifyContent="center"
          >
            {/* Add take profit / stop loss to edit icon below */}
            <EditIcon />
            <Grid item>
              {/* <Button variant="standard" style={{maxHeight:30, width:75}} sx={{ml:1, mb: 1}}>
                trigger
              </Button> */}
              <Button
                variant="outlined"
                style={{ maxHeight: 30, width: 75 }}
                sx={{ ml: 1, mb: 1 }}
                onClick={() => {
                  setOpenClose(true)
                }}
              >
                Limit
              </Button>
              <Button
                variant="contained"
                style={{ maxHeight: 30, width: 75 }}
                sx={{ color: 'background.component', ml: 1, mb: 1 }}
              >
                Market
              </Button>
            </Grid>
            <Grid>
              {/* <TextField
              InputProps={{ style:{height: 30, width:75}}}
              sx={{ml: 1}}
              style={{maxWidth:100}}
              label={'size'}
              defaultValue={value.size}
            /> */}

              {/* <Grid item >
              <Button variant="outlined" sx={{ml: 1}} style={{maxHeight:30}}>
                Limit
              </Button>
            </Grid>
            <Grid item >
              <TextField
                InputProps={{ style:{height: 30}}}
                sx={{ml: 1}}
                style={{maxWidth:100}}
                label={'price'}
                defaultValue={value.size}
              />
            </Grid> */}
            </Grid>
          </Grid>
        ),
      },
    ],
    [settingsState],
  )
  const isLoadingMarketData = false
  // const { data: selectedMarketData } = useSelectMarketDataType(type,  {refetchInterval: 1000})
  // TODO - why do I need to wrap in || [] for useSelectMarketDataType result?
  const data = useMemo(() => DUMMY_POSITIONS || [], [DUMMY_POSITIONS])

  const { getTableProps, headerGroups, prepareRow, rows } = useTable({
    columns,
    data,
    //autoResetPage: false,
  })
  const onRowClick = (
    event: React.MouseEvent<HTMLTableCellElement, MouseEvent>,
    row: Row<Position>,
    column: string,
  ) => {
    return
  }

  const [tab, setTab] = useState(0)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }

  return (
    <>
      <CloseModal openModal={true} setOpenModal={setOpenClose} />
      <StyledCard>
        <TableContainer>
          <Table stickyHeader aria-label="sticky table" {...getTableProps()}>
            <StyledTableHead>
              {headerGroups.map((headerGroup) => {
                const { key: headerGroupKey, ...getHeaderGroupProps } =
                  headerGroup.getHeaderGroupProps()
                return (
                  <TableRow key={headerGroupKey} {...getHeaderGroupProps}>
                    {headerGroup.headers.map((column) => {
                      const { key: headerKey, ...getHeaderProps } =
                        column.getHeaderProps()
                      return (
                        <StyledTableCell key={headerKey} {...getHeaderProps}>
                          <Grid container direction="row" alignItems="center">
                            <Grid item>{column.render('Header')}</Grid>
                          </Grid>
                        </StyledTableCell>
                      )
                    })}
                  </TableRow>
                )
              })}
            </StyledTableHead>
            <TableBody>
              {isLoadingMarketData ? (
                <TableRow>
                  <TableCell>
                    <Skeleton width={50} />
                  </TableCell>
                  {Array(5)
                    .fill(0)
                    .map((x) => (
                      <TableCell key={x}>
                        <Box
                          width={150}
                          textAlign="right"
                          justifyContent="flex-end"
                          display="flex"
                        >
                          <Skeleton width={50} />
                        </Box>
                      </TableCell>
                    ))}
                </TableRow>
              ) : (
                rows.map((row: Row<Position>) => {
                  prepareRow(row)
                  const { key: rowKey, ...rowProps } = row.getRowProps()
                  return (
                    <TableRow
                      key={rowKey}
                      {...rowProps}
                      //style={{ cursor: 'pointer' }}
                      //hover
                    >
                      {row.cells.map((cell: Cell<Position>) => {
                        const { key: cellKey, ...cellProps } =
                          cell.getCellProps()
                        return (
                          <TableCell
                            key={cellKey}
                            {...cellProps}
                            onClick={(event) => {
                              onRowClick(
                                event,
                                row.original,
                                cell.column.Header,
                              )
                            }}
                          >
                            {cell.render('Cell')}
                          </TableCell>
                        )
                      })}
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
        </TableContainer>
      </StyledCard>
    </>
  )
}
