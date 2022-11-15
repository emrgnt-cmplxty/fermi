// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Test basic coin transfer

//# init --accounts A B C

//# view-object 100

//# run sui::pay::split_and_transfer --type-args sui::sui::SUI --args object(100) 10 @B --sender A

//# view-object 100

//# view-object 106

//# run sui::pay::split_and_transfer --type-args sui::sui::SUI --args object(100) 0 @C --sender B

//# view-object 100
