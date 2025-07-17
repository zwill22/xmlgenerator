This fixed the problem of elements being defined after being referenced but has not fixed the problem with only including the first element. 
For example, 
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">

  <xs:element name="name" type="xs:string"/>
  <xs:element name="age" type="xs:positiveInteger"/>
  <xs:element name="height" type="xs:double"/>

  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element ref="name"/>
        <xs:element ref="age"/>
        <xs:element ref="height"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>

</xs:schema>
```
should return an result such as
```xml
<person>
  <name>Continue air recently news whom.</name>
  <age>908</age>
  <height>810.96</height>
</person>
```
but instead returns
```xml
<name>Itself safe television same.</name>
```
There are also some minor issues such as:
- Elements which may occur more than once only occur once in the final output 
- Empty elements are formatted as `<element />` rather than `<element></element>`

