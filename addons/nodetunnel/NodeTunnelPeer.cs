using Godot;
using Godot.Collections;


//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// NodeTunnel by curtjs https://github.com/curtjs/nodetunnel/
// NodeTunnel C# bridge by shysolocup :>

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


#nullable enable


namespace NodeTunnel;


/// <summary>
/// Connection states for tracking relay connection progress
/// </summary>
public enum ConnectionState
{
    Disconnected,
    Connecting,
    Connected,
    Hosting,
    Joined
}


#region Bridge

/// <summary>
/// Extension bridge between normal <see cref="MultiplayerPeer"/>s and a <see cref="NodeTunnelPeer"/>  
/// </summary>
/// 
public static class NodeTunnelBridge
{
    public static NodeTunnelPeer ToNodeTunnelPeer(this MultiplayerPeer self)
    {
        return new NodeTunnelPeer(self);
    }
}

#endregion

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


/// <summary>
/// A relay-based multiplayer peer that connects through NodeTunnel servers.
/// <para/>
/// <para/>Provides P2P-style multiplayer networking through a relay server without requiring direct connections between clients. 
/// <para/>Integrates with Godot's <see cref="MultiplayerApi"/> system
/// </summary>

public partial class NodeTunnelPeer
{
    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region Setup

    /// <summary>
    /// Base <see cref="MultiplayerPeer"/> 
    /// </summary>
    public MultiplayerPeer Peer { get; set; }


    /// <summary>
    /// <para>
    /// If <c>peer</c> is given it'll use that MulitplayerPeer instead of creating a new one
    /// </summary>
    /// <param name="peer">If given it'll use that as the Peer instead of creating a new one</param>
    public NodeTunnelPeer(MultiplayerPeer? peer = null)
    {
        if (peer is null)
        {
            // load the script and make a new one if peer is not given
            var script = GD.Load<GDScript>("res://addons/nodetunnel/NodeTunnelPeer.gd");
            Peer = (MultiplayerPeer)script.New();
        }
        else
            Peer = peer;

        Peer.Connect(SignalName.RelayConnected, CallableRelayConnected);
        Peer.Connect(SignalName.Hosting, CallableHosting);
        Peer.Connect(SignalName.Joined, CallableJoined);
        Peer.Connect(SignalName.RoomLeft, CallableRoomLeft);
    }


    private Callable CallableRelayConnected => Callable.From( (string onlineId) => RelayConnected?.Invoke(onlineId));
    private Callable CallableHosting => Callable.From( () => Hosting?.Invoke());
    private Callable CallableJoined => Callable.From( () => Joined?.Invoke());
    private Callable CallableRoomLeft => Callable.From( () => RoomLeft?.Invoke());


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
     
    #region PropertyName


    public static class PropertyName
    {
        public static readonly StringName RelayHost = new("relay_host");
        public static readonly StringName RelayPort = new("relay_port");
        public static readonly StringName OnlineId = new("online_id");
        public static readonly StringName ConnectionState = new("connection_state");
        public static readonly StringName UniqueId = new("unique_id");
        public static readonly StringName ConnectedPeers = new("connected_peers");
        public static readonly StringName _ConnectionStatus = new("connection_status");
        public static readonly StringName TargetPeer = new("target_peer");
        public static readonly StringName CurrentTransferMode = new("current_transfer_mode");
        public static readonly StringName CurrentTransferChannel = new("current_transfer_channel");
        public static readonly StringName DebugEnabled = new("debug_enabled");
    }


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region SignalName


    public static class SignalName
    {
        public static readonly StringName RelayConnected = new("relay_connected");
        public static readonly StringName Hosting = new("hosting");
        public static readonly StringName Joined = new("joined");
        public static readonly StringName RoomLeft = new("room_left");
    }


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    
    #region MethodName


    public static class MethodName
    {
        public static readonly StringName ConnectToRelay = new("connect_to_relay");
        public static readonly StringName Host = new("host");
        public static readonly StringName Join = new("join");
        public static readonly StringName LeaveRoom = new("leave_room");
        public static readonly StringName DisconnectFromRelay = new("disconnect_from_relay");
        public static readonly StringName IsServer = new("_is_server");
        public static readonly StringName GetUniqueId = new("_get_unique_id");
        public static readonly StringName GetTransferChannel = new("_get_transfer_channel");
        public static readonly StringName SetTransferChannel = new("_set_transfer_channel");
        public static readonly StringName GetTransferMode = new("_get_transfer_mode");
        public static readonly StringName SetTransferMode = new("_set_transfer_mode");
    }


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region Public signals


    /// <summary>
    /// Fires when <see cref="ConnectToRelay"/> succeeds. 
    /// </summary>
    /// <param name="onlineId">The online ID from NodeTunnel</param>
    public event RelayConnectedEventHandler? RelayConnected;
    public delegate void RelayConnectedEventHandler(string onlineId);


    public SignalAwaiter WaitUntilRelayConnected() => Peer.ToSignal(Peer, SignalName.RelayConnected);
    public SignalAwaiter WaitUntilJoined() => Peer.ToSignal(Peer, SignalName.Joined);
    public SignalAwaiter WaitUntilHosting() => Peer.ToSignal(Peer, SignalName.Hosting);
    public SignalAwaiter WaitUntilRoomLeft() => Peer.ToSignal(Peer, SignalName.RoomLeft);


    /// <summary>
    /// Fires when <see cref="Host"/> succeeds. 
    /// </summary>
    public event HostingEventHandler? Hosting;
    public delegate void HostingEventHandler();
    
    
    /// <summary>
    /// Fires when <see cref="Join"/> succeeds. 
    /// </summary>
    public event JoinedEventHandler? Joined;
    public delegate void JoinedEventHandler();
    
    
    /// <summary>
    /// Fires when this peer leaves a room.
    /// <para/>Also fires when the room host leaves and kicks this peer from the room.
    /// </summary>
    public event RoomLeftEventHandler? RoomLeft;
    public delegate void RoomLeftEventHandler();


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region Methods


    /// <summary>
    /// Connect to a NodeTunnel relay server.
    /// </summary>
    /// <param name="address">The relay server hostname or IP address.</param>
    /// <param name="port">The TCP port for the relay server (typically 9998)</param>
    public void ConnectToRelay(string address, int port)
        => Call(MethodName.ConnectToRelay, address, port);


    /// <summary>
    /// Start hosting a multiplayer session.
    /// </summary>
    public void Host()
        => Call(MethodName.Host);


    /// <summary>
    /// Join a multiplayer session using the host's online ID.
    /// </summary>
    /// <param name="hostId">The online ID of the hosting peer.</param>
    public void Join(string hostId)
        => Call(MethodName.Join, hostId);


    /// <summary>
    /// Leave the current multiplayer session.
    /// </summary>
    public void LeaveRoom()
        => Call(MethodName.LeaveRoom);


    /// <summary>
    /// Disconnect from the relay server and clean up all connections.
    /// </summary>
    public void DisconnectFromRelay()
        => Call(MethodName.DisconnectFromRelay);


    public MultiplayerPeer.TransferModeEnum GetTransferMode()
        => Call<MultiplayerPeer.TransferModeEnum>(MethodName.GetTransferMode);

    public void SetTransferMode(MultiplayerPeer.TransferModeEnum mode)
        => Call(MethodName.SetTransferMode, (int)mode);

    public int GetTransferChannel()
        => Call<int>(MethodName.GetTransferChannel);

    public void SetTransferChannel(int channel)
        => Call(MethodName.SetTransferChannel, channel);


    /// <summary>
    /// Get a property from <see cref="Peer"/>.
    /// </summary>
    public Variant Get(StringName propertyName)
        => Peer.Get(propertyName);


    /// <summary>
    /// Get a property from <see cref="Peer"/>.
    /// </summary>
    public T Get<[MustBeVariant] T>(StringName propertyName)
        => Get(propertyName).As<T>();


    /// <summary>
    /// Set a property from <see cref="Peer"/>.
    /// </summary>
    public void Set(StringName propertyName, Variant value)
        => Peer.Set(propertyName, value);


    /// <summary>
    /// Call a method from <see cref="Peer"/>.
    /// </summary>
    public Variant Call(StringName methodName, params Variant[] args)
        => Peer.Call(methodName, args);


    /// <summary>
    /// Call a method from <see cref="Peer"/>.
    /// </summary>
    public T Call<[MustBeVariant] T>(StringName methodName, params Variant[] args)
        => Call(methodName, args).As<T>();


    /// <summary>
    /// Call a method from <see cref="Peer"/> during idle time, always returns null apparently.
    /// </summary>
    public Variant CallDeferred(StringName methodName, params Variant[] args)
        => Peer.CallDeferred(methodName, args);


    /// <summary>
    /// Call a method from <see cref="Peer"/> during idle time, always returns null apparently.
    /// </summary>
    public T CallDeferred<[MustBeVariant] T>(StringName methodName, params Variant[] args)
        => CallDeferred(methodName, args).As<T>();


    /// <summary>
    /// Checks if the peer is the server
    /// </summary>
    public bool IsServer()
        => Call<bool>(MethodName.IsServer);


    /// <summary>
    /// Gets the <see cref="MultiplayerApi"/> unique id 
    /// </summary>
    public int GetUniqueId()
        => Call<int>(MethodName.GetUniqueId);
    


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region Connection configuration


    /// <summary>
    /// Host address to relay from (eg: <c>relay.nodetunnel.io</c>)
    /// </summary>
    public string RelayHost
    {
        get => Get<string>(PropertyName.RelayHost);
        set => Set(PropertyName.RelayHost, value);
    }


    /// <summary>
    /// Port for the address to relay from (eg: <c>9998</c>)
    /// </summary>
    public int RelayPort
    {
        get => Get<int>(PropertyName.RelayPort);
        set => Set(PropertyName.RelayPort, value);
    }


    /// <summary>
    /// String ID for the peer.
    /// </summary>
    public string OnlineId
    {
        get => Get<string>(PropertyName.OnlineId);
    }


    /// <summary>
    /// Current connection state (Disconnected, Connecting, ConnectedHosting, Joined).
    /// </summary>
    public ConnectionState ConnectionState
    {
        get => Get<ConnectionState>(PropertyName.ConnectionState);
    }


    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    #region Multiplayer peer state


    /// <summary>
    /// <see cref="MultiplayerApi"/> unique id for the peer.
    /// </summary>
    public int UniqueId
    {
        get => Get<int>(PropertyName.UniqueId);
    }


    /// <summary>
    /// Dictionary of connected peers.
    /// </summary>
    public Dictionary<int, bool> ConnectedPeers {
        get => Get<Dictionary<int, bool>>(PropertyName.ConnectedPeers);
    }


    /// <summary>
    /// <see cref="MultiplayerApi"/> connection status. 
    /// </summary>
    public MultiplayerPeer.ConnectionStatus _ConnectionStatus
    {
        get => Get<MultiplayerPeer.ConnectionStatus>(PropertyName._ConnectionStatus);
    }


    /// <summary>
    /// debug mode cool
    /// </summary>
    public bool DebugEnabled
    {
        get => Get<bool>(PropertyName.DebugEnabled);
        set => Set(PropertyName.DebugEnabled, value);
    }



    #endregion

    //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
}