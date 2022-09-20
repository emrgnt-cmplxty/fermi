import ReplayIcon from '@mui/icons-material/Replay'
import {
  Box,
  Card,
  Grid,
  Skeleton,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Typography,
} from '@mui/material'
import { PaginationFooter } from 'components/PaginationFooter'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo, useState } from 'react'
import { Cell, Column, Row, usePagination, useTable } from 'react-table'
import { OrderData } from 'utils/globals'

interface OrderHistoryProps {
  input: any
  header: string
}

const OrderHistory = ({ input, header }: OrderHistoryProps) => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)

  const columns: Column<OrderData>[] = useMemo(
    () => [
      {
        Header: 'Time',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">
              {new Date(value.time).toLocaleString()}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Symbol',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{value.marketSymbol}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Side',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid
            sx={{
              color:
                value.side === 'Buy' ? 'tradeColors.bid' : 'tradeColors.ask',
            }}
          >
            <Typography variant="h7">{value.side}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Type',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{value.type}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Trigger Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{value.triggerPrice}</Typography>
          </Grid>
        ),
      },
      //   {
      //     Header: 'Limit Price',
      //     accessor: (row) => {
      //       return row
      //     },
      //     Cell: ({ value }: { value: OrderData }) => (
      //       <Grid>
      //         {value.limitPrice}
      //       </Grid>
      //     ),
      //   },
      {
        Header: 'Avg. Fill Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{value.avgPrice}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Filled/Total',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{`${value.filled}/`}</Typography>
            <Typography color="textSecondary" variant="h7">
              {`${value.size}`}{' '}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Reduce Only',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{`${value.reduce}`}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Post Only',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{`${value.post}`}</Typography>
          </Grid>
        ),
      },
      {
        Header: 'Status',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <ReplayIcon sx={{ mt: 0.5, mb: -0.5 }} />
            <Typography
              variant="h7"
              textColor={
                value.status === 'Canceled' ? 'textSecondary' : 'textSecondary'
              }
            >
              {value.status}
            </Typography>
          </Grid>
        ),
      },
    ],
    [settingsState],
  )
  const isLoadingMarketData = false
  // const { data: selectedMarketData } = useSelectMarketDataType(type,  {refetchInterval: 1000})
  // TODO - why do I need to wrap in || [] for useSelectMarketDataType result?
  const data = useMemo(() => input || [], [input])

  const {
    getTableProps,
    headerGroups,
    prepareRow,
    page,
    canNextPage,
    canPreviousPage,
    pageCount,
    nextPage,
    previousPage,
    gotoPage,
    state: { pageIndex },
  } = useTable(
    {
      columns,
      data,
      initialState: { pageSize: 5 },
      //autoResetPage: false,
    },
    usePagination,
  )
  const onRowClick = (
    event: React.MouseEvent<HTMLTableCellElement, MouseEvent>,
    row: Row<OrderData>,
    column: string,
  ) => {
    return
  }

  const [tab, setTab] = useState(0)
  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setTab(newValue)
  }

  return (
    <Card sx={{ backgroundColor: 'background.component' }}>
      <TableContainer>
        <Table stickyHeader aria-label="sticky table" {...getTableProps()}>
          <TableHead sx={{ backgroundColor: 'background.component' }}>
            {headerGroups.map((headerGroup) => {
              const { key: headerGroupKey, ...getHeaderGroupProps } =
                headerGroup.getHeaderGroupProps()
              return (
                <TableRow key={headerGroupKey} {...getHeaderGroupProps}>
                  {headerGroup.headers.map((column) => {
                    const { key: headerKey, ...getHeaderProps } =
                      column.getHeaderProps()
                    return (
                      <TableCell
                        sx={{ backgroundColor: 'background.component' }}
                        key={headerKey}
                        {...getHeaderProps}
                      >
                        <Grid
                          container
                          direction="row"
                          alignItems="center"
                          sx={{ mt: -1, mb: -1 }}
                        >
                          <Grid item>
                            <Typography variant="h7" color="textSecondary">
                              {column.render('Header')}
                            </Typography>
                          </Grid>
                        </Grid>
                      </TableCell>
                    )
                  })}
                </TableRow>
              )
            })}
          </TableHead>
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
              page.map((row: Row<OrderData>) => {
                prepareRow(row)
                const { key: rowKey, ...rowProps } = row.getRowProps()
                return (
                  <TableRow
                    key={rowKey}
                    {...rowProps}
                    //style={{ cursor: 'pointer' }}
                    //hover
                  >
                    {row.cells.map((cell: Cell<OrderData>) => {
                      const { key: cellKey, ...cellProps } = cell.getCellProps()
                      return (
                        <TableCell
                          key={cellKey}
                          {...cellProps}
                          onClick={(event) => {
                            onRowClick(event, row.original, cell.column.Header)
                          }}
                        >
                          <Grid sx={{ mt: -0.5, mb: -0.5 }}>
                            {cell.render('Cell')}
                          </Grid>
                        </TableCell>
                      )
                    })}
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
        {pageCount > 1 && (
          <PaginationFooter
            page={pageIndex + 1}
            gotoPage={gotoPage}
            nextPage={nextPage}
            previousPage={previousPage}
            canNextPage={canNextPage}
            canPreviousPage={canPreviousPage}
            pageCount={pageCount}
          />
        )}
      </TableContainer>
    </Card>
  )
}

export default OrderHistory
