# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [api.proto](#api-proto)
    - [BlockParent](#massa-api-v1-BlockParent)
    - [BlockResult](#massa-api-v1-BlockResult)
    - [DatastoreEntriesQuery](#massa-api-v1-DatastoreEntriesQuery)
    - [DatastoreEntry](#massa-api-v1-DatastoreEntry)
    - [DatastoreEntryFilter](#massa-api-v1-DatastoreEntryFilter)
    - [EndorsementResult](#massa-api-v1-EndorsementResult)
    - [GetBlocksBySlotsRequest](#massa-api-v1-GetBlocksBySlotsRequest)
    - [GetBlocksBySlotsResponse](#massa-api-v1-GetBlocksBySlotsResponse)
    - [GetDatastoreEntriesRequest](#massa-api-v1-GetDatastoreEntriesRequest)
    - [GetDatastoreEntriesResponse](#massa-api-v1-GetDatastoreEntriesResponse)
    - [GetNextBlockBestParentsRequest](#massa-api-v1-GetNextBlockBestParentsRequest)
    - [GetNextBlockBestParentsResponse](#massa-api-v1-GetNextBlockBestParentsResponse)
    - [GetSelectorDrawsRequest](#massa-api-v1-GetSelectorDrawsRequest)
    - [GetSelectorDrawsResponse](#massa-api-v1-GetSelectorDrawsResponse)
    - [GetTransactionsThroughputRequest](#massa-api-v1-GetTransactionsThroughputRequest)
    - [GetTransactionsThroughputResponse](#massa-api-v1-GetTransactionsThroughputResponse)
    - [GetVersionRequest](#massa-api-v1-GetVersionRequest)
    - [GetVersionResponse](#massa-api-v1-GetVersionResponse)
    - [NewBlocksHeadersRequest](#massa-api-v1-NewBlocksHeadersRequest)
    - [NewBlocksHeadersResponse](#massa-api-v1-NewBlocksHeadersResponse)
    - [NewBlocksRequest](#massa-api-v1-NewBlocksRequest)
    - [NewBlocksResponse](#massa-api-v1-NewBlocksResponse)
    - [NewFilledBlocksRequest](#massa-api-v1-NewFilledBlocksRequest)
    - [NewFilledBlocksResponse](#massa-api-v1-NewFilledBlocksResponse)
    - [NewOperationsFilter](#massa-api-v1-NewOperationsFilter)
    - [NewOperationsQuery](#massa-api-v1-NewOperationsQuery)
    - [NewOperationsRequest](#massa-api-v1-NewOperationsRequest)
    - [NewOperationsResponse](#massa-api-v1-NewOperationsResponse)
    - [OperationResult](#massa-api-v1-OperationResult)
    - [SelectorDraws](#massa-api-v1-SelectorDraws)
    - [SelectorDrawsFilter](#massa-api-v1-SelectorDrawsFilter)
    - [SelectorDrawsQuery](#massa-api-v1-SelectorDrawsQuery)
    - [SendBlocksRequest](#massa-api-v1-SendBlocksRequest)
    - [SendBlocksResponse](#massa-api-v1-SendBlocksResponse)
    - [SendEndorsementsRequest](#massa-api-v1-SendEndorsementsRequest)
    - [SendEndorsementsResponse](#massa-api-v1-SendEndorsementsResponse)
    - [SendOperationsRequest](#massa-api-v1-SendOperationsRequest)
    - [SendOperationsResponse](#massa-api-v1-SendOperationsResponse)
    - [TransactionsThroughputRequest](#massa-api-v1-TransactionsThroughputRequest)
    - [TransactionsThroughputResponse](#massa-api-v1-TransactionsThroughputResponse)
  
    - [OpType](#massa-api-v1-OpType)
  
    - [MassaService](#massa-api-v1-MassaService)
  
- [block.proto](#block-proto)
    - [Block](#massa-api-v1-Block)
    - [BlockHeader](#massa-api-v1-BlockHeader)
    - [FilledBlock](#massa-api-v1-FilledBlock)
    - [FilledOperationTuple](#massa-api-v1-FilledOperationTuple)
    - [SignedBlock](#massa-api-v1-SignedBlock)
    - [SignedBlockHeader](#massa-api-v1-SignedBlockHeader)
  
- [common.proto](#common-proto)
    - [BytesMapFieldEntry](#massa-api-v1-BytesMapFieldEntry)
    - [SecureShare](#massa-api-v1-SecureShare)
  
- [endorsement.proto](#endorsement-proto)
    - [Endorsement](#massa-api-v1-Endorsement)
    - [SignedEndorsement](#massa-api-v1-SignedEndorsement)
  
- [operation.proto](#operation-proto)
    - [CallSC](#massa-api-v1-CallSC)
    - [ExecuteSC](#massa-api-v1-ExecuteSC)
    - [Operation](#massa-api-v1-Operation)
    - [OperationType](#massa-api-v1-OperationType)
    - [RollBuy](#massa-api-v1-RollBuy)
    - [RollSell](#massa-api-v1-RollSell)
    - [SignedOperation](#massa-api-v1-SignedOperation)
    - [Transaction](#massa-api-v1-Transaction)
  
- [slot.proto](#slot-proto)
    - [IndexedSlot](#massa-api-v1-IndexedSlot)
    - [Slot](#massa-api-v1-Slot)
  
- [Scalar Value Types](#scalar-value-types)



<a name="api-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## api.proto



<a name="massa-api-v1-BlockParent"></a>

### BlockParent
Block parent tuple


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| block_id | [string](#string) |  | Block id |
| period | [fixed64](#fixed64) |  | Period |






<a name="massa-api-v1-BlockResult"></a>

### BlockResult
Holds Block response


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| block_id | [string](#string) |  | Block id |






<a name="massa-api-v1-DatastoreEntriesQuery"></a>

### DatastoreEntriesQuery
DatastoreEntries Query


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| filter | [DatastoreEntryFilter](#massa-api-v1-DatastoreEntryFilter) |  | Filter |






<a name="massa-api-v1-DatastoreEntry"></a>

### DatastoreEntry
DatastoreEntry


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| final_value | [bytes](#bytes) |  | final datastore entry value |
| candidate_value | [bytes](#bytes) |  | candidate_value datastore entry value |






<a name="massa-api-v1-DatastoreEntryFilter"></a>

### DatastoreEntryFilter



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| address | [string](#string) |  | Associated address of the entry |
| key | [bytes](#bytes) |  | Datastore key |






<a name="massa-api-v1-EndorsementResult"></a>

### EndorsementResult
Holds Endorsement response


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| endorsements_ids | [string](#string) | repeated | Endorsements ids |






<a name="massa-api-v1-GetBlocksBySlotsRequest"></a>

### GetBlocksBySlotsRequest
GetBlocksBySlotsRequest holds request for GetBlocksBySlots


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| slots | [Slot](#massa-api-v1-Slot) | repeated | Slots |






<a name="massa-api-v1-GetBlocksBySlotsResponse"></a>

### GetBlocksBySlotsResponse
GetBlocksBySlotsResponse holds response from GetBlocksBySlots


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| blocks | [Block](#massa-api-v1-Block) | repeated | Blocks |






<a name="massa-api-v1-GetDatastoreEntriesRequest"></a>

### GetDatastoreEntriesRequest
GetDatastoreEntriesRequest holds request from GetDatastoreEntries


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| queries | [DatastoreEntriesQuery](#massa-api-v1-DatastoreEntriesQuery) | repeated | Queries |






<a name="massa-api-v1-GetDatastoreEntriesResponse"></a>

### GetDatastoreEntriesResponse
GetDatastoreEntriesResponse holds response from GetDatastoreEntries


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| entries | [DatastoreEntry](#massa-api-v1-DatastoreEntry) | repeated | Datastore entries |






<a name="massa-api-v1-GetNextBlockBestParentsRequest"></a>

### GetNextBlockBestParentsRequest
GetNextBlockBestParentsRequest holds request for GetNextBlockBestParents


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-GetNextBlockBestParentsResponse"></a>

### GetNextBlockBestParentsResponse
GetNextBlockBestParentsResponse holds response from GetNextBlockBestParents


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| parents | [BlockParent](#massa-api-v1-BlockParent) | repeated | Best parents |






<a name="massa-api-v1-GetSelectorDrawsRequest"></a>

### GetSelectorDrawsRequest
GetSelectorDrawsRequest holds request from GetSelectorDraws


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| queries | [SelectorDrawsQuery](#massa-api-v1-SelectorDrawsQuery) | repeated | Queries |






<a name="massa-api-v1-GetSelectorDrawsResponse"></a>

### GetSelectorDrawsResponse
GetSelectorDrawsResponse holds response from GetSelectorDraws


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| selector_draws | [SelectorDraws](#massa-api-v1-SelectorDraws) | repeated | Selector draws |






<a name="massa-api-v1-GetTransactionsThroughputRequest"></a>

### GetTransactionsThroughputRequest
GetTransactionsThroughputRequest holds request for GetTransactionsThroughput


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-GetTransactionsThroughputResponse"></a>

### GetTransactionsThroughputResponse
GetTransactionsThroughputResponse holds response from GetTransactionsThroughput


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| throughput | [fixed32](#fixed32) |  | Transactions throughput |






<a name="massa-api-v1-GetVersionRequest"></a>

### GetVersionRequest
GetVersionRequest holds request from GetVersion


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-GetVersionResponse"></a>

### GetVersionResponse
GetVersionResponse holds response from GetVersion


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| version | [string](#string) |  | Version |






<a name="massa-api-v1-NewBlocksHeadersRequest"></a>

### NewBlocksHeadersRequest
NewBlocksHeadersRequest holds request for NewBlocksHeaders


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-NewBlocksHeadersResponse"></a>

### NewBlocksHeadersResponse
NewBlocksHeadersResponse holds response from NewBlocksHeaders


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| block_header | [SignedBlockHeader](#massa-api-v1-SignedBlockHeader) |  | Signed block header |






<a name="massa-api-v1-NewBlocksRequest"></a>

### NewBlocksRequest
NewBlocksRequest holds request for NewBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-NewBlocksResponse"></a>

### NewBlocksResponse
NewBlocksResponse holds response from NewBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| block | [SignedBlock](#massa-api-v1-SignedBlock) |  | Signed block |






<a name="massa-api-v1-NewFilledBlocksRequest"></a>

### NewFilledBlocksRequest
NewFilledBlocksRequest holds request for NewFilledBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |






<a name="massa-api-v1-NewFilledBlocksResponse"></a>

### NewFilledBlocksResponse
NewFilledBlocksResponse holds response from NewFilledBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| filled_block | [FilledBlock](#massa-api-v1-FilledBlock) |  | Block with operations content |






<a name="massa-api-v1-NewOperationsFilter"></a>

### NewOperationsFilter
NewOperations Filter


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| types | [OpType](#massa-api-v1-OpType) | repeated | Operation type enum |






<a name="massa-api-v1-NewOperationsQuery"></a>

### NewOperationsQuery
NewOperations Query


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| filter | [NewOperationsFilter](#massa-api-v1-NewOperationsFilter) |  | Filter |






<a name="massa-api-v1-NewOperationsRequest"></a>

### NewOperationsRequest
NewOperationsRequest holds request for NewOperations


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| query | [NewOperationsQuery](#massa-api-v1-NewOperationsQuery) |  | Query |






<a name="massa-api-v1-NewOperationsResponse"></a>

### NewOperationsResponse
NewOperationsResponse holds response from NewOperations


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| operation | [SignedOperation](#massa-api-v1-SignedOperation) |  | Signed operation |






<a name="massa-api-v1-OperationResult"></a>

### OperationResult
Holds Operation response


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| operations_ids | [string](#string) | repeated | Operations ids |






<a name="massa-api-v1-SelectorDraws"></a>

### SelectorDraws
Selector draws


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| address | [string](#string) |  | Address |
| next_block_draws | [Slot](#massa-api-v1-Slot) | repeated | Next block draws |
| next_endorsement_draws | [IndexedSlot](#massa-api-v1-IndexedSlot) | repeated | Next endorsements draws |






<a name="massa-api-v1-SelectorDrawsFilter"></a>

### SelectorDrawsFilter
SelectorDraws Filter


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| address | [string](#string) |  | Address |






<a name="massa-api-v1-SelectorDrawsQuery"></a>

### SelectorDrawsQuery
SelectorDraws Query


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| filter | [SelectorDrawsFilter](#massa-api-v1-SelectorDrawsFilter) |  | Filter |






<a name="massa-api-v1-SendBlocksRequest"></a>

### SendBlocksRequest
SendBlocksRequest holds parameters to SendBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| block | [SecureShare](#massa-api-v1-SecureShare) |  | Secure shared block |






<a name="massa-api-v1-SendBlocksResponse"></a>

### SendBlocksResponse
SendBlocksResponse holds response from SendBlocks


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| result | [BlockResult](#massa-api-v1-BlockResult) |  | Block result |
| error | [google.rpc.Status](#google-rpc-Status) |  | gRPC error(status) |






<a name="massa-api-v1-SendEndorsementsRequest"></a>

### SendEndorsementsRequest
SendEndorsementsRequest holds parameters to SendEndorsements


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| endorsements | [SecureShare](#massa-api-v1-SecureShare) | repeated | Secure shared endorsements |






<a name="massa-api-v1-SendEndorsementsResponse"></a>

### SendEndorsementsResponse
SendEndorsementsResponse holds response from SendEndorsements


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| result | [EndorsementResult](#massa-api-v1-EndorsementResult) |  | Endorsement result |
| error | [google.rpc.Status](#google-rpc-Status) |  | gRPC error(status) |






<a name="massa-api-v1-SendOperationsRequest"></a>

### SendOperationsRequest
SendOperationsRequest holds parameters to SendOperations


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| operations | [SecureShare](#massa-api-v1-SecureShare) | repeated | Secured shared operations |






<a name="massa-api-v1-SendOperationsResponse"></a>

### SendOperationsResponse
SendOperationsResponse holds response from SendOperations


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| result | [OperationResult](#massa-api-v1-OperationResult) |  | Operation result |
| error | [google.rpc.Status](#google-rpc-Status) |  | gRPC error(status) |






<a name="massa-api-v1-TransactionsThroughputRequest"></a>

### TransactionsThroughputRequest
TransactionsThroughputRequest holds request for TransactionsThroughput


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| interval | [fixed64](#fixed64) | optional | Optional timer interval in sec. Defaults to 10s |






<a name="massa-api-v1-TransactionsThroughputResponse"></a>

### TransactionsThroughputResponse
TransactionsThroughputResponse holds response from TransactionsThroughput


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| id | [string](#string) |  | Request id |
| throughput | [fixed32](#fixed32) |  | Transactions throughput |





 


<a name="massa-api-v1-OpType"></a>

### OpType
Operation type enum

| Name | Number | Description |
| ---- | ------ | ----------- |
| OP_TYPE_UNSPECIFIED | 0 | Defaut enum value |
| OP_TYPE_TRANSACTION | 1 | Transaction |
| OP_TYPE_ROLL_BUY | 2 | Roll buy |
| OP_TYPE_ROLL_SELL | 3 | Roll sell |
| OP_TYPE_EXECUTE_SC | 4 | Execute smart contract |
| OP_TYPE_CALL_SC | 5 | Call smart contract |


 

 


<a name="massa-api-v1-MassaService"></a>

### MassaService
Massa gRPC service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| GetBlocksBySlots | [GetBlocksBySlotsRequest](#massa-api-v1-GetBlocksBySlotsRequest) | [GetBlocksBySlotsResponse](#massa-api-v1-GetBlocksBySlotsResponse) | Get blocks by slots |
| GetDatastoreEntries | [GetDatastoreEntriesRequest](#massa-api-v1-GetDatastoreEntriesRequest) | [GetDatastoreEntriesResponse](#massa-api-v1-GetDatastoreEntriesResponse) | Get datastore entries |
| GetNextBlockBestParents | [GetNextBlockBestParentsRequest](#massa-api-v1-GetNextBlockBestParentsRequest) | [GetNextBlockBestParentsResponse](#massa-api-v1-GetNextBlockBestParentsResponse) | Get next block best parents |
| GetSelectorDraws | [GetSelectorDrawsRequest](#massa-api-v1-GetSelectorDrawsRequest) | [GetSelectorDrawsResponse](#massa-api-v1-GetSelectorDrawsResponse) | Get selector draws |
| GetTransactionsThroughput | [GetTransactionsThroughputRequest](#massa-api-v1-GetTransactionsThroughputRequest) | [GetTransactionsThroughputResponse](#massa-api-v1-GetTransactionsThroughputResponse) | Get transactions throughput |
| GetVersion | [GetVersionRequest](#massa-api-v1-GetVersionRequest) | [GetVersionResponse](#massa-api-v1-GetVersionResponse) | Get node version |
| NewBlocks | [NewBlocksRequest](#massa-api-v1-NewBlocksRequest) stream | [NewBlocksResponse](#massa-api-v1-NewBlocksResponse) stream | New received and produced blocks |
| NewBlocksHeaders | [NewBlocksHeadersRequest](#massa-api-v1-NewBlocksHeadersRequest) stream | [NewBlocksHeadersResponse](#massa-api-v1-NewBlocksHeadersResponse) stream | New received and produced blocks headers |
| NewFilledBlocks | [NewFilledBlocksRequest](#massa-api-v1-NewFilledBlocksRequest) stream | [NewFilledBlocksResponse](#massa-api-v1-NewFilledBlocksResponse) stream | New received and produced blocks with operations |
| NewOperations | [NewOperationsRequest](#massa-api-v1-NewOperationsRequest) stream | [NewOperationsResponse](#massa-api-v1-NewOperationsResponse) stream | New received and produced perations |
| SendBlocks | [SendBlocksRequest](#massa-api-v1-SendBlocksRequest) stream | [SendBlocksResponse](#massa-api-v1-SendBlocksResponse) stream | Send blocks |
| SendEndorsements | [SendEndorsementsRequest](#massa-api-v1-SendEndorsementsRequest) stream | [SendEndorsementsResponse](#massa-api-v1-SendEndorsementsResponse) stream | Send endorsements |
| SendOperations | [SendOperationsRequest](#massa-api-v1-SendOperationsRequest) stream | [SendOperationsResponse](#massa-api-v1-SendOperationsResponse) stream | Send operations |
| TransactionsThroughput | [TransactionsThroughputRequest](#massa-api-v1-TransactionsThroughputRequest) stream | [TransactionsThroughputResponse](#massa-api-v1-TransactionsThroughputResponse) stream | Transactions throughput per second |

 



<a name="block-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## block.proto



<a name="massa-api-v1-Block"></a>

### Block
Block


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| header | [SignedBlockHeader](#massa-api-v1-SignedBlockHeader) |  | Signed header |
| operations | [string](#string) | repeated | Operations ids |






<a name="massa-api-v1-BlockHeader"></a>

### BlockHeader
Block header


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| slot | [Slot](#massa-api-v1-Slot) |  | Slot |
| parents | [string](#string) | repeated | parents |
| operation_merkle_root | [string](#string) |  | All operations hash |
| endorsements | [SignedEndorsement](#massa-api-v1-SignedEndorsement) | repeated | Signed endorsements |






<a name="massa-api-v1-FilledBlock"></a>

### FilledBlock
Filled block


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| header | [SignedBlockHeader](#massa-api-v1-SignedBlockHeader) |  | Signed header |
| operations | [FilledOperationTuple](#massa-api-v1-FilledOperationTuple) | repeated | Operations |






<a name="massa-api-v1-FilledOperationTuple"></a>

### FilledOperationTuple
Filled Operation Tuple


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| operation_id | [string](#string) |  | Operation id |
| operation | [SignedOperation](#massa-api-v1-SignedOperation) |  | Signed operation |






<a name="massa-api-v1-SignedBlock"></a>

### SignedBlock
Signed block


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| content | [Block](#massa-api-v1-Block) |  | Block |
| signature | [string](#string) |  | A cryptographically generated value using `serialized_data` and a public key. |
| content_creator_pub_key | [string](#string) |  | The public-key component used in the generation of the signature |
| content_creator_address | [string](#string) |  | Derived from the same public key used to generate the signature |
| id | [string](#string) |  | A secure hash of the data. See also [massa_hash::Hash] |






<a name="massa-api-v1-SignedBlockHeader"></a>

### SignedBlockHeader
Signed block header


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| content | [BlockHeader](#massa-api-v1-BlockHeader) |  | BlockHeader |
| signature | [string](#string) |  | A cryptographically generated value using `serialized_data` and a public key. |
| content_creator_pub_key | [string](#string) |  | The public-key component used in the generation of the signature |
| content_creator_address | [string](#string) |  | Derived from the same public key used to generate the signature |
| id | [string](#string) |  | A secure hash of the data. See also [massa_hash::Hash] |





 

 

 

 



<a name="common-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## common.proto



<a name="massa-api-v1-BytesMapFieldEntry"></a>

### BytesMapFieldEntry
BytesMapFieldEntry


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| key | [bytes](#bytes) |  | bytes key |
| value | [bytes](#bytes) |  | bytes key |






<a name="massa-api-v1-SecureShare"></a>

### SecureShare
Packages a type such that it can be securely sent and received in a trust-free network


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| serialized_data | [bytes](#bytes) |  | Content in sharable, deserializable form. Is used in the secure verification protocols |
| signature | [string](#string) |  | A cryptographically generated value using `serialized_data` and a public key. |
| content_creator_pub_key | [string](#string) |  | The public-key component used in the generation of the signature |
| content_creator_address | [string](#string) |  | Derived from the same public key used to generate the signature |
| id | [string](#string) |  | A secure hash of the data. See also [massa_hash::Hash] |





 

 

 

 



<a name="endorsement-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## endorsement.proto



<a name="massa-api-v1-Endorsement"></a>

### Endorsement
An endorsement, as sent in the network


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| slot | [Slot](#massa-api-v1-Slot) |  | Slot in which the endorsement can be included |
| index | [fixed32](#fixed32) |  | Endorsement index inside the including block |
| endorsed_block | [string](#string) |  | Hash of endorsed block This is the parent in thread `self.slot.thread` of the block in which the endorsement is included |






<a name="massa-api-v1-SignedEndorsement"></a>

### SignedEndorsement
Signed endorsement


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| content | [Endorsement](#massa-api-v1-Endorsement) |  | Endorsement |
| signature | [string](#string) |  | A cryptographically generated value using `serialized_data` and a public key. |
| content_creator_pub_key | [string](#string) |  | The public-key component used in the generation of the signature |
| content_creator_address | [string](#string) |  | Derived from the same public key used to generate the signature |
| id | [string](#string) |  | A secure hash of the data. See also [massa_hash::Hash] |





 

 

 

 



<a name="operation-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## operation.proto



<a name="massa-api-v1-CallSC"></a>

### CallSC
Calls an exported function from a stored smart contract


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| target_addr | [string](#string) |  | Target smart contract address |
| target_func | [string](#string) |  | Target function name. No function is called if empty |
| param | [bytes](#bytes) |  | Parameter to pass to the target function |
| max_gas | [fixed64](#fixed64) |  | The maximum amount of gas that the execution of the contract is allowed to cost |
| coins | [fixed64](#fixed64) |  | Extra coins that are spent from the caller&#39;s balance and transferred to the target |






<a name="massa-api-v1-ExecuteSC"></a>

### ExecuteSC
Execute a smart contract


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| data | [bytes](#bytes) |  | Smart contract bytecode. |
| max_gas | [fixed64](#fixed64) |  | The maximum amount of gas that the execution of the contract is allowed to cost |
| datastore | [BytesMapFieldEntry](#massa-api-v1-BytesMapFieldEntry) | repeated | A key-value store associating a hash to arbitrary bytes |






<a name="massa-api-v1-Operation"></a>

### Operation
The operation as sent in the network


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| fee | [fixed64](#fixed64) |  | The fee they have decided for this operation |
| expire_period | [fixed64](#fixed64) |  | After `expire_period` slot the operation won&#39;t be included in a block |
| op | [OperationType](#massa-api-v1-OperationType) |  | The type specific operation part |






<a name="massa-api-v1-OperationType"></a>

### OperationType
Type specific operation content


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| transaction | [Transaction](#massa-api-v1-Transaction) |  | Transfer coins from sender to recipient |
| roll_buy | [RollBuy](#massa-api-v1-RollBuy) |  | The sender buys `roll_count` rolls. Roll price is defined in configuration |
| roll_sell | [RollSell](#massa-api-v1-RollSell) |  | The sender sells `roll_count` rolls. Roll price is defined in configuration |
| execut_sc | [ExecuteSC](#massa-api-v1-ExecuteSC) |  | Execute a smart contract |
| call_sc | [CallSC](#massa-api-v1-CallSC) |  | Calls an exported function from a stored smart contract |






<a name="massa-api-v1-RollBuy"></a>

### RollBuy
The sender buys `roll_count` rolls. Roll price is defined in configuration


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| roll_count | [fixed64](#fixed64) |  | Roll count |






<a name="massa-api-v1-RollSell"></a>

### RollSell
The sender sells `roll_count` rolls. Roll price is defined in configuration


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| roll_count | [fixed64](#fixed64) |  | Roll count |






<a name="massa-api-v1-SignedOperation"></a>

### SignedOperation
Signed operation


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| content | [Operation](#massa-api-v1-Operation) |  | Operation |
| signature | [string](#string) |  | A cryptographically generated value using `serialized_data` and a public key. |
| content_creator_pub_key | [string](#string) |  | The public-key component used in the generation of the signature |
| content_creator_address | [string](#string) |  | Derived from the same public key used to generate the signature |
| id | [string](#string) |  | A secure hash of the data. See also [massa_hash::Hash] |






<a name="massa-api-v1-Transaction"></a>

### Transaction
Transfer coins from sender to recipient


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| recipient_address | [string](#string) |  | Recipient address |
| amount | [fixed64](#fixed64) |  | Amount |





 

 

 

 



<a name="slot-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## slot.proto



<a name="massa-api-v1-IndexedSlot"></a>

### IndexedSlot
When an address is drawn to create an endorsement it is selected for a specific index


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| slot | [Slot](#massa-api-v1-Slot) |  | Slot |
| index | [fixed64](#fixed64) |  | Endorsement index in the slot |






<a name="massa-api-v1-Slot"></a>

### Slot
A point in time where a block is expected


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| period | [fixed64](#fixed64) |  | Period |
| thread | [fixed32](#fixed32) |  | Thread |





 

 

 

 



## Scalar Value Types

| .proto Type | Notes | C++ | Java | Python | Go | C# | PHP | Ruby |
| ----------- | ----- | --- | ---- | ------ | -- | -- | --- | ---- |
| <a name="double" /> double |  | double | double | float | float64 | double | float | Float |
| <a name="float" /> float |  | float | float | float | float32 | float | float | Float |
| <a name="int32" /> int32 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint32 instead. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="int64" /> int64 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint64 instead. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="uint32" /> uint32 | Uses variable-length encoding. | uint32 | int | int/long | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="uint64" /> uint64 | Uses variable-length encoding. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum or Fixnum (as required) |
| <a name="sint32" /> sint32 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int32s. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sint64" /> sint64 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int64s. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="fixed32" /> fixed32 | Always four bytes. More efficient than uint32 if values are often greater than 2^28. | uint32 | int | int | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="fixed64" /> fixed64 | Always eight bytes. More efficient than uint64 if values are often greater than 2^56. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum |
| <a name="sfixed32" /> sfixed32 | Always four bytes. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sfixed64" /> sfixed64 | Always eight bytes. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="bool" /> bool |  | bool | boolean | boolean | bool | bool | boolean | TrueClass/FalseClass |
| <a name="string" /> string | A string must always contain UTF-8 encoded or 7-bit ASCII text. | string | String | str/unicode | string | string | string | String (UTF-8) |
| <a name="bytes" /> bytes | May contain any arbitrary sequence of bytes. | string | ByteString | str | []byte | ByteString | string | String (ASCII-8BIT) |

