Complex elements containing multiple elements are generated correctly regardless of where the sub-elements are defined.
However, using type definitions to define an element fails. For example the XSD
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:complexType name="characterType">
        <xs:sequence>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="screentime" type="xs:positiveInteger"/>
        </xs:sequence>
    </xs:complexType>

    <xs:simpleType name="filmidtype">
        <xs:restriction base="xs:string">
            <xs:pattern value="[0-9]{6}"/>
        </xs:restriction>
    </xs:simpleType>

    <xs:complexType name="filmtype">
        <xs:sequence>
            <xs:element name="title" type="xs:string"/>
            <xs:element name="character" maxOccurs="unbounded" type="characterType"/>
        </xs:sequence>
        <xs:attribute name="filmid" type="filmidtype" use="required"/>
    </xs:complexType>

    <xs:element name="film" type="filmtype"/>
</xs:schema>
```
should return something like:

```xml
<?xml version="1.0" encoding="UTF-8" ?>
<film filmid="123456">
    <title>Revenge of the Switch</title>
    <character>
        <name>Only-One Ketchup</name>
        <screentime>100</screentime>
    </character>
    <character>
        <name>Daft Vein</name>
        <screentime>101</screentime>
    </character>
</film>
```
Instead, the code returns:
```xml
<character>
  <name>Left perhaps mind. Unit hope huge only occur.</name>
  <screentime>16</screentime>
</character>
```
Additional issues:
- elements that can occur multiple times are always included only once.
- empty elements render as `<Data />` rather than `<Data></Data>`
- output must begin with the element `<?xml version="1.0" encoding="UTF-8"?>` to be valid XML