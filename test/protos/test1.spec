
package jubjubnest.test

message ObjectId
{
    // Constructs an item info that references an internal object.
    Internal(
        // Object type.
        int32 type,

        // Object ID.
        int32 item_id
    )
    {
        type_id = type
        item_id = ObjectItemId::Internal( item_id )
    }
}

message ObjectItemId
{
    // Constructs an item info that references an internal object.
    Internal( int32 item_id )
    {
        internal_id = item_id
    }
}
