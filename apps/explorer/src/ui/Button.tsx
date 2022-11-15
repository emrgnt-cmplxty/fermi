// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { cva, type VariantProps } from 'class-variance-authority';

import { ButtonOrLink, type ButtonOrLinkProps } from './utils/ButtonOrLink';

const buttonStyles = cva(
    [
        'inline-flex items-center justify-center',
        // TODO: Remove when CSS reset is applied.
        'cursor-pointer no-underline',
    ],
    {
        variants: {
            variant: {
                primary:
                    'bg-sui-dark text-sui-light hover:text-white border-none',
                secondary:
                    'bg-sui-grey-90 text-sui-grey-50 hover:text-white border-none',
                outline:
                    'bg-white border border-solid border-sui-grey-55 text-sui-grey-70 hover:text-sui-grey-90 hover:border-sui-grey-65 active:text-sui-grey-100 active:border-sui-grey-75',
            },
            size: {
                md: 'px-3 py-2 rounded-md text-bodySmall font-semibold',
                lg: 'px-4 py-3 rounded-lg text-body font-semibold',
            },
        },
        defaultVariants: {
            variant: 'primary',
            size: 'md',
        },
    }
);

export interface ButtonProps
    extends VariantProps<typeof buttonStyles>,
        ButtonOrLinkProps {}

export function Button({ variant, size, ...props }: ButtonProps) {
    return (
        <ButtonOrLink className={buttonStyles({ variant, size })} {...props} />
    );
}
