const Component: React.Component<{}> = () => {
  return (
    <Wrapper prop={markedFunction('prop')}>
      <div>
        {markedFunction('child')}
      </div>
    </Wrapper>
  );
}