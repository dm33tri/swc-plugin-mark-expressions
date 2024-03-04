const Component: React.Component<{
}> = ()=>{
    return <Wrapper prop={markedFunction(/* markExpression: prop */ 'prop')}>

      <div>

        {markedFunction(/* markExpression: child */ 'child')}

      </div>

    </Wrapper>;
};
