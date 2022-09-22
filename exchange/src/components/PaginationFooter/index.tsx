import {
  FirstPage,
  LastPage,
  NavigateBefore,
  NavigateNext,
} from '@mui/icons-material'
import { Box, IconButton, Typography } from '@mui/material'

export interface PaginationFooterProps {
  gotoPage: any
  nextPage: any
  previousPage: any
  canNextPage: boolean
  canPreviousPage: boolean
  pageCount: number
  page: number
}

export const PaginationFooter = ({
  gotoPage,
  nextPage,
  previousPage,
  canNextPage,
  canPreviousPage,
  pageCount,
  page,
}: PaginationFooterProps) => {
  return (
    <Box
      display="flex"
      justifyContent="center"
      alignItems="center"
      sx={{ width: '100%' }}
    >
      <IconButton disabled={!canPreviousPage} onClick={() => gotoPage(0)}>
        <FirstPage />
      </IconButton>
      <IconButton disabled={!canPreviousPage} onClick={() => previousPage()}>
        <NavigateBefore />
      </IconButton>
      <Typography>{`Page ${page} of ${pageCount}`}</Typography>
      <IconButton disabled={!canNextPage} onClick={() => nextPage()}>
        <NavigateNext />
      </IconButton>
      <IconButton
        disabled={!canNextPage}
        onClick={() => gotoPage(pageCount - 1)}
      >
        <LastPage />
      </IconButton>
    </Box>
  )
}
