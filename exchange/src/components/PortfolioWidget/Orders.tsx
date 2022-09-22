import DeleteIcon from '@mui/icons-material/Delete'
import EditIcon from '@mui/icons-material/Edit'
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
import { AssetDisplay } from 'components/AssetDisplay'
import { PaginationFooter } from 'components/PaginationFooter'
import { useUserData } from 'hooks/react-query/useUser'
import { SettingsContext } from 'providers/SettingsProvider'
import { useContext, useMemo } from 'react'
import { Cell, Column, Row, usePagination, useTable } from 'react-table'
import { OrderData } from 'utils/globals'

interface OrdersProps {
  header: string
}

const Orders = ({ header }: OrdersProps) => {
  const { settingsState, settingsDispatch, isLoading } =
    useContext(SettingsContext)
  const { data: userData } = useUserData({ refetchInterval: 1000 })

  const columns: Column<OrderData>[] = useMemo(
    () => [
      {
        Header: 'Contracts',
        maxWidth: 200,
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            {
              <AssetDisplay
                symbol={value.baseSymbol}
                rightLabel={`${String(value.baseSymbol)}${
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
        Header: 'Price',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">{value.price}</Typography>
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
            <Typography variant="h7">
              {`${value.size} / ${value.filled}`}
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
      // {
      //   Header: 'Post Only',
      //   accessor: (row) => {
      //     return row
      //   },
      //   Cell: ({ value }: { value: OrderData }) => (
      //     <Grid>
      //       <Typography variant="h7">
      //         {`${value.post}`}
      //       </Typography>
      //     </Grid>
      //   ),
      // },
      {
        Header: 'Order Time	',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <Typography variant="h7">
              {`${new Date(value.time).toLocaleString()}`}
            </Typography>
          </Grid>
        ),
      },
      {
        Header: 'Modify',
        accessor: (row) => {
          return row
        },
        Cell: ({ value }: { value: OrderData }) => (
          <Grid>
            <EditIcon />
            <DeleteIcon />
          </Grid>
        ),
      },
    ],
    [settingsState],
  )
  const isLoadingMarketData = false
  // const { data: selectedMarketData } = useSelectMarketDataType(type,  {refetchInterval: 1000})
  // TODO - why do I need to wrap in || [] for useSelectMarketDataType result?
  const data = useMemo(
    () => userData?.openLimitOrders || [],
    [userData?.openLimitOrders],
  )

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

export default Orders
