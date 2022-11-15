// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import { useMemo, useEffect, useCallback, useState } from 'react';
import { Navigate, useSearchParams, useNavigate } from 'react-router-dom';

import { SuiIcons } from '_components/icon';
import Loading from '_components/loading';
import Overlay from '_components/overlay';
import ReceiptCard from '_components/receipt-card';
import { useAppSelector, useAppDispatch } from '_hooks';
import { getTransactionsByAddress } from '_redux/slices/txresults';

import type { TxResultState } from '_redux/slices/txresults';

import st from './ReceiptPage.module.scss';

// Response pages for all transactions
// use txDigest for the transaction result
function ReceiptPage() {
    const [searchParams] = useSearchParams();
    const [showModal, setShowModal] = useState(true);
    const dispatch = useAppDispatch();
    // get tx results from url params
    const txDigest = searchParams.get('txdigest');

    const transferType = searchParams.get('transfer') as 'nft' | 'coin';

    const txResults: TxResultState[] = useAppSelector(
        ({ txresults }) => txresults.latestTx
    );

    const loading: boolean = useAppSelector(
        ({ txresults }) => txresults.loading
    );

    useEffect(() => {
        dispatch(getTransactionsByAddress()).unwrap();
    }, [dispatch]);

    const txnItem = useMemo(() => {
        return txResults.filter((txn) => txn.txId === txDigest)[0];
    }, [txResults, txDigest]);

    //TODO: redo the CTA links
    const ctaLinks = transferType === 'nft' ? '/nfts' : '/';
    const linkTo = transferType ? ctaLinks : '/transactions';

    const navigate = useNavigate();
    const closeReceipt = useCallback(() => {
        navigate(linkTo);
    }, [linkTo, navigate]);

    if ((!txDigest && !txnItem) || (!loading && !txResults.length)) {
        return <Navigate to={linkTo} replace={true} />;
    }

    const callMeta =
        txnItem?.name && txnItem?.url ? 'Minted Successfully!' : 'Move Call';

    const transferLabel =
        txnItem?.kind === 'Call'
            ? 'Call'
            : txnItem?.isSender
            ? 'Sent'
            : 'Received';
    //TODO : add more transfer types and messages
    const transfersTxt = {
        Call: callMeta || 'Call',
        Sent: 'Successfully Sent!',
        Received: 'Successfully Received!',
    };

    const kind = txnItem?.kind as keyof typeof transfersTxt | undefined;

    const headerCopy = kind ? transfersTxt[transferLabel] : '';

    const transferStatus =
        txnItem?.status === 'success'
            ? headerCopy
            : txnItem?.status
            ? 'Transaction Failed'
            : '';

    return (
        <Loading loading={loading} className={st.centerLoading}>
            <Overlay
                showModal={showModal}
                setShowModal={setShowModal}
                title={transferStatus}
                closeOverlay={closeReceipt}
                closeIcon={SuiIcons.Check}
                className={st.receiptOverlay}
            >
                {txnItem && <ReceiptCard txDigest={txnItem} />}
            </Overlay>
        </Loading>
    );
}

export default ReceiptPage;
