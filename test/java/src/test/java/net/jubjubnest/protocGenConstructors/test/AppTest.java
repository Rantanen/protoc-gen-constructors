package net.jubjubnest.protocGenConstructors.test;

import static org.junit.Assert.assertEquals;

import org.junit.Test;
import net.jubjubnest.Protos;

public class AppTest 
{
    @Test
    public void shouldSetInternalIds()
    {
        Protos.ObjectId objid = Protos.ObjectId.internal(1, 2);
        assertEquals(1, objid.getTypeId());
        assertEquals(2, objid.getItemId().getInternalId());
    }
}
