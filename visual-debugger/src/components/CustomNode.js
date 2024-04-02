import { Handle, Position } from 'reactflow';

const CustomNode = ({ data, id}) => {

  return (
    <div onDoubleClick={() => data.onChildAction(data.label, data.parent)} style={{ borderStyle: 'solid', height: '100%',	display: 'flex',	alignItems: 'center'}} >
    {Array.from({ length: data.inputHandles }).map((_, index) => (
        <Handle
          type="target"
          position={Position.Top}
          id={`${data.label}.in[${index}]`}
          key={`${data.label}.in[${index}]`}
          style={{ left: `calc(33% * ${index + 1})` }}
        />
      ))}
      {/* <Handle type="target" position={Position.Top} /> */}
      <label htmlFor="text" style={{overflow: 'hidden', width: '100%'}}>{data.label}</label>
      <Handle type="source" position={Position.Bottom}  id={`${data.label}.out`}
          key={`${data.label}.out`}/>
    </div>
  );
};
  export default CustomNode;
