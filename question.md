The code works for a single complex element containing multiple simple elements.
If the complex element contains another complex element, this works so long as the complex sub-element is defined inside the parent.
For example,
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
        <xs:element name="stats">
          <xs:complexType>
            <xs:sequence>
              <xs:element name="height" type="xs:double"/>
              <xs:element name="weight" type="xs:double"/>
            </xs:sequence>
          </xs:complexType>
        </xs:element>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>
```
However, if the sub-element is referenced from the parent element, e.g.
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="stats">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="height" type="xs:double"/>
                <xs:element name="weight" type="xs:double"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
    
    <xs:element name="person">
        <xs:complexType>
            <xs:sequence>
                <xs:element name="name" type="xs:string"/>
                <xs:element name="age" type="xs:integer"/>
                <xs:element ref="stats"/>
            </xs:sequence>
        </xs:complexType>
    </xs:element>
</xs:schema>
```
the code returns only a fake `stats` element.
Additionally, the definition of an element referenced from another complex element could be before or after the parent. For example
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
works correctly, but
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element ref="name"/>
        <xs:element ref="age"/>
        <xs:element ref="height"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>

  <xs:element name="name" type="xs:string"/>
  <xs:element name="age" type="xs:positiveInteger"/>
  <xs:element name="height" type="xs:double"/>
</xs:schema>
```
does not. In this case, the code generates a random string for each element reference in `person` regardless of the type, e.g.
```xml
<person>
  increase
  successful
  vote
</person>
```
Similarly, for two complex elements, as in
```xml
<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="name" type="xs:string"/>
        <xs:element name="age" type="xs:integer"/>
        <xs:element ref="stats"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>

  <xs:element name="stats">
    <xs:complexType>
      <xs:sequence>
        <xs:element name="height" type="xs:double"/>
        <xs:element name="weight" type="xs:double"/>
      </xs:sequence>
    </xs:complexType>
  </xs:element>
</xs:schema>
```
the code returns a correct `name` and `age` but returns a random string for `stats`:
```xml
<person>
  <name>Law prevent kid first. Top drug store through.</name>
  <age>433</age>
  reality
</person>
```
