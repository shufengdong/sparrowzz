
import org.jgrapht.UndirectedGraph
import org.jgrapht.graph.SimpleGraph
import org.jgrapht.graph.Subgraph
import zju.devmodel.MapObject
import zju.dsmodel.DistriSys
import zju.dsmodel.DsDevices
import zju.dsmodel.DsModelCons
import zju.dsmodel.DsTopoNode
import zju.measure.DiscreteInfo
import zju.measure.MeasureInfo
import java.util.*

/**
 * 缙云电网的cim文件有以下几个问题:
 * 1. 没有TransformerWinding，而PowerTransformer被看成是ConductingEquipment，具有两个端子
 * 2. 没有发电机,在判断是否是有源是通过是否有变压器设备代替
 * 3. 电源点无法确定
 * @date 2021/6/30.
 */
class JyPowerSystem(val sourceStationNames : Array<String>) : PowerSystem() {

    var passed : Boolean = true
    var errorId : String? = null

    inner class JyMeasureProvider : MeasureProvider {
        override fun getAnalogValue(measId: String, dataSource: String): Double? {
            throw UnsupportedOperationException("not implemented") //To change body of created functions use File | Settings | File Templates.
        }

        override fun getDiscreteValue(measId: String, dataSource: String): Int? {
            throw UnsupportedOperationException("not implemented") //To change body of created functions use File | Settings | File Templates.
        }

        override fun getAnalogsOfEquip(equip: String, dataSource: String): List<MeasureInfo> {
            throw UnsupportedOperationException("not implemented") //To change body of created functions use File | Settings | File Templates.
        }

        override fun getDiscreteOfEquip(equip: String, dataSource: String): DiscreteInfo {
            throw UnsupportedOperationException("not implemented") //To change body of created functions use File | Settings | File Templates.
        }

        override fun setResourceManager(resourceManager: PowerSystem) {
            throw UnsupportedOperationException("not implemented") //To change body of created functions use File | Settings | File Templates.
        }

        override fun getSwitchStatus(switchId: String, dataSource: String): Boolean {
            val dev = getResource(switchId)
            val r = dev?.getProperty("NORMALOPEN")?.toBoolean()
            if (r == null)
                return true
            else
                return r
        }
    }

    init {
        this.measureProvider = JyMeasureProvider()
    }

    constructor() : this(emptyArray())

    fun getNodeType(v: TN): Int {
        var nodeType = DsTopoNode.TYPE_LINK
        for (cn in v.cns) {
            if (sourceCns.contains(cn))
                return DsTopoNode.TYPE_SUB
            val shuntEquips = getShuntEquips(cn)
            if (!shuntEquips.isEmpty()) {
                assert(shuntEquips.size == 1 && (shuntEquips[0].type == "ENERGYCONSUMER" || shuntEquips[0].type == "POWERTRANSFORMER"))
                nodeType = DsTopoNode.TYPE_PQ
            }
        }
        return nodeType
    }

    val simpledIslands = LinkedList<UndirectedGraph<TN, Edge>>()
    val activeIslands = LinkedList<UndirectedGraph<String, Edge>>()

    override fun filterDevices() {
        try {
            if (sourceCns.size == 0) {
                // 搜索电源相关的CN
                val substations = classToList["SUBSTATION"]?.filter { it.getProperty("NAME") != null && sourceStationNames.contains(it.getProperty("NAME").substringAfter("kV")) }
                assert(substations?.size == sourceStationNames.size)
                val substationIds = LinkedList<String>()
                substations!!.forEach { substationIds.add(it.id) }
    //            val sourceBus = classToList["BUSBARSECTION"]?.filter { substationIds.contains(it.getProperty("MEMBEROF_EQUIPMENTCONTAINER")) }
                val sourceBus = classToList["BUSBARSECTION"]?.filter { substationIds.contains(it.getProperty("EQUIPMENTCONTAINER")) }
                sourceBus!!.forEach {
                    if (getTerminalsOfEquip(it).isEmpty()) {
                        passed = false
                        errorId = it.id
                        return
                    }
                    val cn = getCNOfTerminal(getTerminalsOfEquip(it)[0])!!.id
                    if (!sourceCns.contains(cn))
                        sourceCns.add(cn)
                }
            }
            super.filterDevices()
        } catch (e : Exception) {
            e.printStackTrace()
            return
        }
    }

    override fun extraFilterDevices(graphes : MutableList<Subgraph<String, Edge, *>>) {
        var aclineNum = 0
        var switchNum = 0
        var transformerNum = 0
        var i = 1
        for (g in graphes) {
            var aclineNum2 = 0
            var switchNum2 = 0
            var transformerNum2 = 0
            val cnInGraph = g.vertexSet().filterNot { it.endsWith(suffixOfAddedCn) }
            cnInGraph.forEach { obj -> getTerminalsOfCN(getResource(obj)!!).map { getEquipOfTerminal(it) }.forEach {
                if (it?.type == "ACLINESEGMENT") {
                    if (getTerminalsOfEquip(it!!).size == 1) {
                        aclineNum2 += 2
                    } else {
                        aclineNum2++
                    }
                } else if (it?.type == "POWERTRANSFORMER") {
                    if (getTerminalsOfEquip(it!!).size == 1) {
                        transformerNum2 += 2
                    } else {
                        transformerNum2++
                    }
                } else if (it != null && isSwitch(it)) {
                    if(getTerminalsOfEquip(it).size == 1) {
                        switchNum2 += 6
                    } else if(getTerminalsOfEquip(it).size == 3) {
                        switchNum2 += 2
                    } else {
                        switchNum2 += 3
                    }
                }
            }}
            aclineNum2 /= 2
            switchNum2 /= 6
            transformerNum2 /= 2
            aclineNum += aclineNum2
            switchNum += switchNum2
            transformerNum += transformerNum2

            println("====================== " + i++ + " th island information =======================")
            println("There are $aclineNum2 ac lines.")
            println("There are $switchNum2 switches.")
            println("There are $transformerNum2 transformers.")
        }

        aclines = HashMap<String, MapObject>(aclineNum)
        switches = HashMap<String, MapObject>(switchNum)
        transformers = HashMap<String, MapObject>(transformerNum)
        for (g in graphes)
            g.vertexSet().filterNot { it.endsWith(suffixOfAddedCn) }.forEach { cn -> getTerminalsOfCN(getResource(cn)!!)
                    .mapNotNull { getEquipOfTerminal(it) }
                    .forEach {
                        if(it.type == "ACLINESEGMENT") {
                            if (!aclines!!.containsKey(it.id))
                                aclines!!.put(it.id, it)
                        } else if(it.type == "POWERTRANSFORMER") {
                            if (!transformers!!.containsKey(it.id))
                                transformers!!.put(it.id, it)
                        } else if (isSwitch(it)) {
                            if (!switches!!.containsKey(it.id))
                                switches!!.put(it.id, it)
                        }
                    }
            }
    }

    fun createActiveIslands() {
        for (island in activeSubgraphes) {
            val g = SimpleGraph<String, Edge>(Edge::class.java)
            island.vertexSet().forEach {
                g.addVertex(it)
            }
            island.edgeSet().forEach {
                g.addEdge(island.getEdgeSource(it), island.getEdgeTarget(it), it)
            }
            activeIslands.add(g)
        }
    }

    /**
     * 生成计算模型
     */
    fun createCalMode(): List<DistriSys> {
        val result = ArrayList<DistriSys>(simpledIslands.size)
        try {
            var j = 1
            // 网络简化之后的结果
            for (island in simpledIslands) {
                val devices = DsDevices()
                devices.initialLists()

                val vertexSet = island.vertexSet().toList()
                for (i in 1..vertexSet.size) {
                    val v = vertexSet[i - 1]
                    if (getNodeType(v) == DsTopoNode.TYPE_PQ) {
                        v.id = "Load_$i" // 设置id
                        // 开始填充设备
                        val load = MapObject()
                        load.id = v.cns[0]
                        load.setProperty(DsModelCons.KEY_CONNECTED_NODE, v.id)
                        devices.spotLoads.add(load)
                    } else if (getNodeType(v) == DsTopoNode.TYPE_SUB) {
                        v.id = "SRC_$i" // 设置id
                    } else
                        v.id = "LINK_$i" // 设置id
                }
                // 处理边
                island.edgeSet().forEach {
                    val tn1 = island.getEdgeSource(it)
                    val tn2 = island.getEdgeTarget(it)
                    // 支路只有开关类型
                    val sw = MapObject()
                    sw.id = tn1.id + "-" + tn2.id
                    sw.setProperty(DsModelCons.KEY_RESOURCE_TYPE, DsModelCons.RESOURCE_SWITCH)
                    sw.setProperty(DsModelCons.KEY_CONNECTED_NODE, tn1.id + ";" + tn2.id)
                    devices.switches.add(sw)
                }
                val r = DistriSys()
                r.buildOrigTopo(devices)
                result.add(r)
                j++
            }
        } catch (e : Exception) {
            e.printStackTrace()
        }
        return result
    }

    /**
     * 简化网络
     */
    fun doSimplification() {
        try {
            transformers!!.values.forEach { assert(getTerminalsOfEquip(it).size == 2) }
//        origGraph.edgeSet().forEach {
//            if(it.equips.size != 1)
//                print("hh")
//            assert(it.equips.size == 1)
//        }
            origGraph.edgeSet().forEach {
                if (getResource(it.equips[0]) != null)
                    assert(isSwitch(getResource(it.equips[0])!!) || isConnEquip(getResource(it.equips[0])))
                else
                    assert(isSwitch(getResource(it.equips[0].substring(0, it.equips[0].length - 3))!!))
            }
            for (island in activeSubgraphes) {
                val cnToTN = HashMap<String, TN>(island.vertexSet().size)
                val g = SimpleGraph<TN, Edge>(Edge::class.java)
                island.vertexSet().forEach {
                    val tn = TN()
                    tn.cns.add(it)
                    cnToTN[it] = tn
                    g.addVertex(tn)
                }
                island.edgeSet().forEach {
                    val edge = Edge()
                    edge.equips.addAll(it.equips)
                    g.addEdge(cnToTN[island.getEdgeSource(it)], cnToTN[island.getEdgeTarget(it)], edge)
                }
                // todo: 去除没有电源的负荷

                // 合并所有连在一起的AclineSegment
                val feederEdges = g.edgeSet().filter { it.equips.all { isConnEquip(getResource(it)) } }
                feederEdges.forEach {
                    if (g.edgeSet().contains(it)) {
                        val v1 = g.getEdgeSource(it)
                        val v2 = g.getEdgeTarget(it)
                        mergeTwoVertex(g, v1, v2)
                    }
                }

                //找出来开关对应的支路
//                val switchEdges = g.edgeSet().filter {
//                    it.equips.size == 1 && (isSwitch(it.equips[0])
//                            || isSwitch(it.equips[0].substring(0, it.equips[0].length - 3)))
//                }
//                for (e in switchEdges) {
//                    if (!g.containsEdge(e))
//                        continue
//                    val v1 = g.getEdgeSource(e)
//                    val v2 = g.getEdgeTarget(e)
//                    if (v1 == null || v2 == null)
//                        continue
//                    g.removeEdge(e)
//                    val inspector = ConnectivityInspector(g)
//                    val graphs = inspector.connectedSets() //获得连通子图
//                    // 如果网络没有分裂
//                    if (graphs.size != 2) {
//                        g.addEdge(v1, v2, e)
//                        continue
//                    }
//                    // 网络分裂成两个, 下面两种情况需要处理
//                    // 1. 其中之一是无源的包含负荷的子网,合并节点
//                    // 2. 其中之一是无源的不包含负荷的子网,删除即可
//                    var needAdd = true
//                    for (subV in graphs) {
//                        var isNoSource = true
//                        var isNoLoad = true
//                        subV.map { getNodeType(it) }.forEach {
//                            if (it == DsTopoNode.TYPE_DG || it == DsTopoNode.TYPE_SUB) {
//                                isNoSource = false
//                                return@forEach
//                            } else if (it == DsTopoNode.TYPE_PQ) {
//                                isNoLoad = false
//                            }
//                        }
//                        if (isNoSource) {
//                            if (!isNoLoad) {//需要合并
//                                g.addEdge(v1, v2, e)
//                                mergeTwoVertex(g, v1, v2)
//                                needAdd = false
//                                break
//                            } else {//删除子图
////                                subV.forEach { g.removeVertex(it) }
////                                needAdd = false
////                                break
//                            }
//                        }
//                    }
//                    if (needAdd)
//                        g.addEdge(v1, v2, e)
//                }

                // 删除只和两条支路连接的LinkNode
                var isFinish = false
                while (!isFinish) {
                    isFinish = true
                    val linkNodes = g.vertexSet().filter { getNodeType(it) == DsTopoNode.TYPE_LINK && g.edgesOf(it).size == 2 }
                    linkNodes.forEach {
                        val edges = g.edgesOf(it).toList()
                        val v1: TN
                        val v2: TN
                        v1 = if (g.getEdgeSource(edges[0]) == it)
                            g.getEdgeTarget(edges[0])
                        else
                            g.getEdgeSource(edges[0])
                        v2 = if (g.getEdgeSource(edges[1]) == it)
                            g.getEdgeTarget(edges[1])
                        else
                            g.getEdgeSource(edges[1])
                        g.removeVertex(it)
                        val edge = Edge()
                        // 将所有设备都与该边联系在一起
                        edge.equips.addAll(edges[0].equips)
                        edge.equips.addAll(edges[1].equips)
                        g.addEdge(v1, v2, edge)
                        isFinish = false
                    }
                }
                simpledIslands.add(g)
            }
        } catch (e : Exception) {
            e.printStackTrace()
        }
    }

    private fun mergeTwoVertex(g: SimpleGraph<TN, Edge>, v1: TN, v2: TN) {
        val edgesOfV2 = g.edgesOf(v2).toList()
        edgesOfV2.forEach {
            val anotherV: TN = if (v2 == g.getEdgeSource(it)) {
                g.getEdgeTarget(it)
            } else
                g.getEdgeSource(it)
            g.removeEdge(it)
            if (v1 == anotherV)
                return@forEach

            val e = g.getEdge(v1, anotherV)
            if (e != null) {
                if(it.equips.all { isConnEquip(getResource(it)) })
                    return@forEach
                else
                    e.equips.addAll(it.equips)
            } else
                g.addEdge(v1, anotherV, it)
        }
        v1.cns.addAll(v2.cns)
        g.removeVertex(v2)
    }

    override fun getShuntEquips(cn: MapObject): List<MapObject> {
        val l = LinkedList<MapObject>()
        getTerminalsOfCN(cn).forEach {
            val equip = getEquipOfTerminal(it)
            if (equip != null) {
                if (shuntEquipTypes.contains(equip.type) || equip.type == "POWERTRANSFORMER")
                    l.add(equip)
            }
        }
        return l
    }

    fun clearSimpledIslands() {
        simpledIslands.clear()
    }

    fun isConnEquip(obj : MapObject?) : Boolean {
        return obj?.type == "ACLINESEGMENT" || obj?.type == "CONNLINE" || obj?.type == "POWERTRANSFORMER"
    }
}
