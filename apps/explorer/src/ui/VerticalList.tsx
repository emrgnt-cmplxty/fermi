// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import clsx from 'clsx';
import { type ReactNode } from 'react';

export interface ListItemProps {
    active?: boolean;
    children: ReactNode;
    onClick?(): void;
}

export function ListItem({ active, children, onClick }: ListItemProps) {
    return (
        <li className="list-none">
            <button
                type="button"
                className={clsx(
                    'cursor-pointer px-3 py-2 rounded-md text-body block w-full border-1 border-solid text-left',
                    active
                        ? 'bg-sui-grey-45 text-sui-grey-90 font-semibold border-sui-grey-50 shadow-sm'
                        : 'bg-white text-sui-grey-80 font-medium border-transparent'
                )}
                onClick={onClick}
            >
                {children}
            </button>
        </li>
    );
}

export interface VerticalListProps {
    children: ReactNode;
}

export function VerticalList({ children }: VerticalListProps) {
    return <ul className="flex flex-col p-0 m-0 gap-1">{children}</ul>;
}
