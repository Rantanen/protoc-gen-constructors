
require('mocha');
const expect = require('chai').expect;

const test1 = require('../build/test1_pb');
const test2 = require('../build/test2_pb');

require('../build/test1_pb-constructors');
require('../build/test2_pb-constructors');

describe('ObjectId', () => {
    describe('#internal()', () => {
        it('should work', () => {

            let objid = test1.ObjectId.internal(1,2);

            expect(objid.getTypeId()).to.equal(1);
            expect(objid.getItemId().getInternalId()).to.equal(2);
        });
    });
});
