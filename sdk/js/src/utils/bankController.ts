import { CreateAssetRequest, PaymentRequest } from '../../dist/proto/bank_requests_pb'

export function buildPaymentRequest(receiver: Uint8Array, assetId: number, quantity: number): PaymentRequest {
  const paymentRequest = new PaymentRequest()
  paymentRequest.setReceiver(receiver)
  paymentRequest.setAssetId(assetId)
  paymentRequest.setQuantity(quantity)

  return paymentRequest
}

export function buildCreateAssetRequest(dummy: number): CreateAssetRequest {
  const createAssetRequest = new CreateAssetRequest()
  createAssetRequest.setDummy(dummy)

  return createAssetRequest
}
