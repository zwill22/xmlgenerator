Currently, the code includes a return statement
```python
if depth > 10:
    return
```
which prevents an infinite recursive loop.
However, if a schema does include such a loop, the code should throw an error. For example, 
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="Data">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="DataRecord" maxOccurs="unbounded" type="DataRecordType"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>

    <xs:complexType name="DataRecordType">
        <xs:sequence>
            <xs:element ref="Data"/>
        </xs:sequence>
    </xs:complexType>

</xs:schema>
```
currently returns 
```xml
<?xml version="1.0" encoding="UTF-8"?>
<DataRecord>
  <Data>
    <DataRecord>
      <Data>
        <DataRecord>
          <Data />
        </DataRecord>
        <DataRecord>
          <Data />
        </DataRecord>
      </Data>
    </DataRecord>
    <DataRecord>
      <Data>
        <DataRecord>
          <Data />
        </DataRecord>
        <DataRecord>
          <Data />
        </DataRecord>
      </Data>
    </DataRecord>
  </Data>
</DataRecord>
```
Instead, an error should be raised.