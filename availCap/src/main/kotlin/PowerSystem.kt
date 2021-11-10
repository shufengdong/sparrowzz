
import org.jgrapht.UndirectedGraph
import org.jgrapht.alg.ConnectivityInspector
import org.jgrapht.graph.SimpleGraph
import org.jgrapht.graph.Subgraph
import org.xml.sax.Attributes
import org.xml.sax.SAXException
import org.xml.sax.helpers.DefaultHandler
import zju.devmodel.MapObject
import zju.measure.MeasureInfo
import java.io.InputStream
import java.util.*
import javax.xml.parsers.SAXParserFactory

class TN(var id : String = "", val cns : LinkedList<String> = LinkedList())
class Edge(val equips : LinkedList<String> = LinkedList(), var limI : Double = 1000.0, var type : Int = 1)

/**
 * 该类用于管理电网资源(MapObject)的方法
 * @date 17-1-8.
 */
open class PowerSystem {

    companion object {
        val switchTypes = arrayOf("BREAKER", "DISCONNECTOR", "FUSE", "GROUNDDISCONNECTOR", "COMPOSITESWITCH",
                "JUMPER", "LOADBREAKSWITCH")
        val conductingEquipTypes = arrayOf("ACLINESEGMENT", "TRANSFORMERWINDING", "BUSBARSECTION", "ENERGYCONSUMER",
                "SYNCHRONOUSMACHINE", "SERIESCOMPENSATOR", "SHUNTCOMPENSATOR", "DCLINESEGMENT", "JUNCTION", "CONNLINE",
                "POWERTRANSFORMER")
        val shuntEquipTypes = arrayOf("ENERGYCONSUMER", "SYNCHRONOUSMACHINE", "SHUNTCOMPENSATOR")
        val suffixOfAddedCn = "-neutral"
        val minIslandSize = 5
    }

    val classToList = HashMap<String, MutableList<MapObject>>()
    private var idToObject: MutableMap<String, MapObject> = HashMap()
//    val assetIdToObject: MutableMap<String, MapObject> = HashMap()
    val poleNameToCn: MutableMap<String, String> = HashMap()

    // 主要的电力系统资源
    var generators: MutableMap<String, MapObject>? = null
    var loads: MutableMap<String, MapObject>? = null
    var busbars: MutableMap<String, MapObject>? = null
    var shuntCompensators: MutableMap<String, MapObject>? = null
    var seriesCompensators: MutableMap<String, MapObject>? = null
    var aclines: MutableMap<String, MapObject>? = null
    var connlines: MutableMap<String, MapObject>? = null
    var windings: MutableMap<String, MapObject>? = null
    var switches: MutableMap<String, MapObject>? = null
    var transformers: MutableMap<String, MapObject>? = null
    // 需要的CN
    var cns: MutableMap<String, MapObject>? = null
    // 量测
    var analogs: MutableMap<String, MapObject>? = null
    var discretes: MutableMap<String, MapObject>? = null
    var equipMeasurements: MutableMap<String, List<MeasureInfo>>? = null
    //过滤量测和拓扑分析时需要用到该类
    var measureProvider : MeasureProvider? = null
    //量测来源
    var dataSource = CimConstants.MEASURE_VALUE_SOURCE_SCADA
    // 表明电力系统资源是否添加完毕,一旦完毕无法在添加
    var isInitial = false
    // 原始的有源拓扑岛
    val origGraph = SimpleGraph<String, Edge>(Edge::class.java)
    val activeSubgraphes: MutableList<Subgraph<String, Edge, *>> = LinkedList()

    // 电源所在的ConnectivityNode
    var sourceCns = LinkedList<String>()

    fun getResourceByClassId(aClass: String): List<MapObject> {
        classToList.putIfAbsent(aClass, ArrayList<MapObject>(0))
        return classToList[aClass]!!
    }

    fun getResource(obj: String): MapObject? {
        return idToObject[obj]
    }

    /**
     * 从CIM XML中导入电网资源
     */
    fun loadFromCimXML(input : InputStream) {
        // step 1: 获得SAX解析器工厂实例
        val factory = SAXParserFactory.newInstance()
        // step 2: 获得SAX解析器实例
        val parser = factory.newSAXParser()
        // step 3: 开始进行解析
        // 传入待解析的文档的处理器
        parser.parse(input, CimXmlHandler())
    }

    /**
     * 添加设备,所有设备添加结束之后应该调用endAddPSR方法
     */
    fun addPSR(obj : MapObject) {
        if(isInitial)
            throw Exception("Power system is already initialed, new resource can not be added anymore.")
        if(obj.id == null || obj.id.trim() == "") {
            System.err.println("Illegal id of power system resource, id can not be null, this obj will not be added.")
            System.err.println(obj.toString())
            return
        }
        val isExist = idToObject.containsKey(obj.id)
        if(!isExist) { // 同一个对象不能addPSR两次,否则这里添加不进去
            idToObject.put(obj.id, obj)
            if (obj.type == null || obj.type.trim() == "") {
                print("!!No type is found for power system resource whose id is ${obj.id}")
                return
            }
            if (!classToList.containsKey(obj.type))
                classToList.put(obj.type, LinkedList<MapObject>())
            classToList[obj.type]!!.add(obj)
//            if (obj.getProperty("ASSETS") != null)
//                assetIdToObject[obj.getProperty("ASSETS")] = obj
        }
    }

    open fun endAddPSR() {
        if(isInitial)
            throw Exception("Power system is already initialed.")
        isInitial = true
        // 处理容器
        idToObject.values.forEach {
            val keys = it.properties.keys.filter { it.startsWith("MEMBEROF_") }
            if(keys.size == 1) {
                val r = idToObject[it.properties[keys[0]]]
                if(r != null)
                    it.containerId = r.id
            }
        }
        // 处理Terminal和ConnectivityNode之间的关系
        val keys = arrayOf("CONDUCTINGEQUIPMENT","CONNECTIVITYNODE")
        classToList["TERMINAL"]?.forEach {
            // 处理1对多的关系
            for(key in keys) {
                val r = idToObject[it.properties[key]]
                if (r != null) {
                    val s = r.properties[it.type]
                    if (s == null)
                        r.properties[it.type] = it.id
                    else
                        r.properties[it.type] = s + ";" + it.id
                }
            }
        }
        // 处理TransformerWinding与PowerTransformer之间的关系
        classToList["TRANSFORMERWINDING"]?.forEach {
            // 处理1对多的关系
            val r = idToObject[it.properties["MEMBEROF_POWERTRANSFORMER"]]
            if (r != null) {
                val s = r.properties[it.type]
                if (s == null)
                    r.properties[it.type] = it.id
                else
                    r.properties[it.type] = s + ";" + it.id
            }
        }
        // 结束之后开始过滤没有必要分析的设备
        filterDevices()
    }

    private fun getObjectList(obj : MapObject, propertyName : String) : List<MapObject> {
        val terminals = obj.properties[propertyName]
        val l = LinkedList<MapObject>()
        if(terminals != null) {
            terminals.split(";").forEach {
                val t = idToObject[it]
                if(t != null)
                    l.add(t)
            }
        }
        return l
    }

    fun getShuntEquips(cn : String) : List<MapObject> {
        val node = getResource(cn)
        if(node == null) return LinkedList()
        else return getShuntEquips(node)
    }

    open fun getShuntEquips(cn : MapObject) : List<MapObject> {
        val l = LinkedList<MapObject>()
        getTerminalsOfCN(cn).forEach {
            val equip = getEquipOfTerminal(it)
            if(equip != null) {
                if(shuntEquipTypes.contains(equip.type))
                    l.add(equip)
            }
        }
        return l
    }

    fun getTerminalsOfEquip(obj : MapObject) : List<MapObject> {
        return getObjectList(obj, "TERMINAL")
    }

    fun getCNOfTerminal(t: MapObject) : MapObject? {
        val cnId = t.getProperty("CONNECTIVITYNODE")
        if(cnId == null) {
            println("!!No connectivy id of terminal ${t.id} is found.")
            return null
        } else
            return getResource(cnId)
    }

    fun getTerminalsOfCN(cn : MapObject) : List<MapObject> {
        return getObjectList(cn, "TERMINAL")
    }

    fun getEquipOfTerminal(t : MapObject) : MapObject? {
        return getResource(t.getProperty("CONDUCTINGEQUIPMENT"))
    }

    fun getWindingsOfTf(transformer: MapObject) : List<MapObject> {
        return getObjectList(transformer, "TRANSFORMERWINDING")
    }

    fun getTfOfWinding(winding : MapObject) : MapObject? {
        return getResource(winding.getProperty("MEMBEROF_POWERTRANSFORMER"))
    }

    fun isSwitch(id : String) : Boolean {
        val sw = getResource(id)
        return sw != null && isSwitch(sw)
    }

    fun isSwitch(obj : MapObject) : Boolean {
        return switchTypes.any { obj.type.toUpperCase() == it.toUpperCase() }
    }

    fun getAllSwitch() : List<MapObject> {
        val r = LinkedList<MapObject>()
        switchTypes.mapNotNull { classToList[it] }.forEach { r.addAll(it) }
        return r
    }

    open fun getConductingEquips() : MutableList<MapObject> {
        val r = LinkedList<MapObject>()
        switchTypes.mapNotNull { classToList[it] }.forEach { r.addAll(it) }
        conductingEquipTypes.mapNotNull { classToList[it] }.forEach { r.addAll(it) }
        return r
    }

    fun isActiveCn(cn : String) : Boolean {
        if(cn.endsWith(suffixOfAddedCn))
            return false
        getTerminalsOfCN(getResource(cn)!!)
                // 判断是否是有电源的
                .filter { getEquipOfTerminal(it)?.type == "SYNCHRONOUSMACHINE" }
                .forEach { return true }
        return false
    }

    fun isActive(subVertexes : Set<String>) : Boolean {
        if(subVertexes.size < minIslandSize)
            return false
        @Suppress("LoopToCallChain")
        for (cn in subVertexes) {
            if(sourceCns.contains(cn))
                return true
        }
        return false
    }

    open fun buildGraph() : MutableList<Subgraph<String, Edge, *>> {
        try {
            // 清空所有定点
            origGraph.vertexSet().toList().forEach { origGraph.removeVertex(it) }
            for (obj in getResourceByClassId("CONNECTIVITYNODE"))
                origGraph.addVertex(obj.id)
            for (obj in getConductingEquips()) {
                val terminals = getTerminalsOfEquip(obj)
                if (terminals.size != 2)
                    continue
                if (isSwitch(obj) and (obj.getProperty("OPEN") == "true"))
                    continue
                val cn1 = getCNOfTerminal(terminals[0])
                val cn2 = getCNOfTerminal(terminals[1])
                if (cn1 == null || cn2 == null || cn1.id == cn2.id)
                    continue
                addEdge(cn1.id, cn2.id, obj.id, origGraph)
            }
            // 处理杆和cn的关系
            for (obj in getResourceByClassId("POLECODE")) {
                val terminals = getTerminalsOfEquip(obj)
                if (terminals.size != 1)
                    continue
                val cn = getCNOfTerminal(terminals[0]) ?: continue
                if (obj.getProperty("NAME") != null)
                    poleNameToCn[obj.getProperty("NAME")] = cn.id
            }
            for (obj in getResourceByClassId("POWERTRANSFORMER")) {
                val windings = getWindingsOfTf(obj)
                if (windings.size < 2)
                    continue
                val magneticCn = MapObject() // 中性点对应的CN
                magneticCn.id = obj.id + suffixOfAddedCn
                magneticCn.type = "CONNECTIVITYNODE"
                origGraph.addVertex(magneticCn.id)
                windings.forEach {
                    val cn = getCNOfTerminal(getTerminalsOfEquip(it)[0])
                    addEdge(magneticCn.id, cn!!.id, it.id, origGraph)
                }
            }
            extraBuildGraph()
            // 连通性分析
            val inspector = ConnectivityInspector(origGraph)
            val graphs = inspector.connectedSets() //获得连通子图

            // 找出有电源的CN
            graphs.forEach({
                it.filter { isActiveCn(it) }.forEach({ sourceCns.add(it) })
            })

            // 找出有源的电气岛
            graphs.filter { isActive(it) }
                    .mapTo(activeSubgraphes) { Subgraph<String, Edge, UndirectedGraph<String, Edge>>(origGraph, it) }
            println("========== There are " + graphs.size + " islands totally. ============")
            println("========== There are " + activeSubgraphes.size + " active islands. ============")
            for (i in 1..activeSubgraphes.size) {
                val vertexNum = activeSubgraphes[i - 1].vertexSet().size
                val edgeNum = activeSubgraphes[i - 1].edgeSet().size
                println("========== There are $vertexNum vertexes in sub island $i.============")
                println("========== There are $edgeNum edges in sub island $i.============")
            }
        } catch (e: Exception) {
            e.printStackTrace();
        }
        return activeSubgraphes
    }

    open fun extraBuildGraph() {}

    fun addEdge(cn1: String, cn2: String, obj: String, g: SimpleGraph<String, Edge>) {
        val e = g.getEdge(cn1, cn2)
        if (e == null) {
            val edge = Edge()
            edge.equips.add(obj)
            g.addEdge(cn1, cn2, edge)
        } else
            e.equips.add(obj)
    }

    /**
     * 形成原始的拓扑图(node-brake),并过滤掉无源的拓扑岛
     */
    open fun filterDevices() {
        val graphs = buildGraph() //先分析原始的拓扑图
        filterDevices(graphs)
    }

    open fun filterDevices(graphs: MutableList<Subgraph<String, Edge, *>>) {
//        val graphs = activeSubgraphes //先分析原始的拓扑图
        // 统计ConnectivityNode的数量
        val cnNum = graphs.sumBy { it.vertexSet().size }
        cns = HashMap(cnNum)
        for(g in graphs)
            g.vertexSet().forEach {
                if(!it.endsWith(suffixOfAddedCn))
                    cns!!.put(it, getResource(it)!!)
            }
        var generatorNum = 0
        var loadNum = 0
        var busbarNum = 0
        var shuntCompensatorNum = 0
        var seriesCompensatorNum = 0
        var aclineNum = 0
        var connlineNum = 0
        var windingNum = 0
        var switchNum = 0
        var i = 1
        for (g in graphs) {
            var generatorNum2 = 0
            var loadNum2 = 0
            var busbarNum2 = 0
            var shuntCompensatorNum2 = 0
            var seriesCompensatorNum2 = 0
            var aclineNum2 = 0
            var connlineNum2 = 0
            var windingNum2 = 0
            var switchNum2 = 0
            val cnInGraph = g.vertexSet().filterNot { it.endsWith(suffixOfAddedCn) }
            cnInGraph.forEach { obj -> getTerminalsOfCN(getResource(obj)!!).map { getEquipOfTerminal(it) }.forEach {
                if (it?.type == "SYNCHRONOUSMACHINE")
                    generatorNum2++
                else if (it?.type == "ENERGYCONSUMER")
                    loadNum2++
                else if (it?.type == "BUSBARSECTION")
                    busbarNum2++
                else if (it?.type == "SHUNTCOMPENSATOR")
                    shuntCompensatorNum2++
                else if (it?.type == "SERIESCOMPENSATOR")
                    seriesCompensatorNum2++
                else if (it?.type == "ACLINESEGMENT")
                    aclineNum2++
                else if (it?.type == "CONNLINE")
                    connlineNum2++
                else if (it?.type == "TRANSFORMERWINDING")
                    windingNum2++
                else if (it != null && isSwitch(it))
                    switchNum2++
            }}
            aclineNum2 /= 2
            connlineNum2 /= 2
            seriesCompensatorNum2 /= 2
            switchNum2 /= 2

            generatorNum += generatorNum2
            loadNum += loadNum2
            busbarNum += busbarNum2
            shuntCompensatorNum += shuntCompensatorNum2
            seriesCompensatorNum += seriesCompensatorNum2
            aclineNum += aclineNum2
            connlineNum += connlineNum2
            windingNum += windingNum2
            switchNum += switchNum2

            println("====================== " + i++ + " th island information =======================")
            println("There are $generatorNum2 generators.")
            println("There are $loadNum2 loads.")
            println("There are $busbarNum2 bus bar sections.")
            println("There are $shuntCompensatorNum2 shunt compensators.")
            println("There are $seriesCompensatorNum2 series compensators.")
            println("There are $aclineNum2 ac lines.")
            println("There are $connlineNum2 conn lines.")
            println("There are $windingNum2 transformer winding.")
            println("There are $switchNum2 switches.")
        }

        generators = HashMap<String, MapObject>(generatorNum)
        loads = HashMap<String, MapObject>(loadNum)
        busbars = HashMap<String, MapObject>(busbarNum)
        shuntCompensators = HashMap<String, MapObject>(shuntCompensatorNum)
        seriesCompensators = HashMap<String, MapObject>(seriesCompensatorNum)
        aclines = HashMap<String, MapObject>(aclineNum)
        connlines = HashMap<String, MapObject>(connlineNum)
        windings = HashMap<String, MapObject>(windingNum)
        transformers = HashMap<String, MapObject>(windingNum / 2)
        switches = HashMap<String, MapObject>(switchNum)
        for (g in graphs)
            g.vertexSet().filterNot { it.endsWith(suffixOfAddedCn) }.forEach { cn -> getTerminalsOfCN(getResource(cn)!!)
                    .mapNotNull { getEquipOfTerminal(it) }
                    .forEach {
                        if (it.type == "SYNCHRONOUSMACHINE")
                            generators!!.put(it.id, it)
                        else if (it.type == "ENERGYCONSUMER")
                            loads!!.put(it.id, it)
                        else if (it.type == "BUSBARSECTION")
                            busbars!!.put(it.id, it)
                        else if (it.type == "SHUNTCOMPENSATOR")
                            shuntCompensators!!.put(it.id, it)
                        else if (it.type == "TRANSFORMERWINDING") {
                            windings!!.put(it.id, it)
                            val transformer = getTfOfWinding(it)
                            if (transformer == null) {
                                System.err.println("Transformer winding does not belong to any transformer, id:" + it.id + " name:" + it.name)
                            } else if (!transformers!!.containsKey(transformer.id))
                                transformers!!.put(transformer.id, transformer)
                        } else if (it.type == "SERIESCOMPENSATOR") {
                            if (!seriesCompensators!!.containsKey(it.id))
                                seriesCompensators!!.put(it.id, it)
                        } else if (it.type == "ACLINESEGMENT") {
                            if (!aclines!!.containsKey(it.id))
                                aclines!!.put(it.id, it)
                        } else if (it.type == "CONNLINE") {
                            if (!connlines!!.containsKey(it.id))
                                connlines!!.put(it.id, it)
                        } else if (isSwitch(it)) {
                            if (!switches!!.containsKey(it.id))
                                switches!!.put(it.id, it)
                        }
                    }
            }
        extraFilterDevices(graphs)

        println("=========== needed generators num: " + generators!!.size)
        println("=========== needed load num: " + loads!!.size)
        println("=========== needed bus bar num: " + busbars!!.size)
        println("=========== needed shunt compensator num: " + shuntCompensators!!.size)
        println("=========== needed series compensator num: " + seriesCompensators!!.size)
        println("=========== needed ac line num: " + aclines!!.size)
        println("=========== needed conn line num: " + connlines!!.size)
        println("=========== needed winding num: " + windings!!.size)
        println("=========== needed transformer num: " + transformers!!.size)
        println("=========== needed switch num: " + switches!!.size)
        println("=========== needed connectivity node num: " + cnNum)
    }

    open fun extraFilterDevices(graphes : MutableList<Subgraph<String, Edge, *>>) {}

    /**
     * 分析需要的量测信息
     */
    open fun filterMeasure() {
        println("Checking measurement start..")
        val start = System.currentTimeMillis()
        analogs = HashMap(6000)//todo:size is not good
        discretes = HashMap(switches!!.size)
        equipMeasurements = HashMap()

        findAnalogs(generators!!.values)
        findAnalogs(loads!!.values)
        findAnalogs(busbars!!.values)
        findAnalogs(shuntCompensators!!.values)
        findAnalogs(seriesCompensators!!.values)
        findAnalogs(aclines!!.values)
        findAnalogs(connlines!!.values)
        findAnalogs(windings!!.values)

        findDiscretes(switches!!.values)

        println("=========== needed analog measurement num: " + analogs!!.size)
        println("=========== needed discrete measurement num: " + discretes!!.size)
        println("Checking measurement end, and time used: " + (System.currentTimeMillis() - start) + "ms")
    }

    //todo:支只处理了开关位置,变压器档位等量测没有处理
    private fun findDiscretes(switches: Collection<MapObject>) {
        switches.mapNotNull { measureProvider?.getDiscreteOfEquip(it.id, dataSource) }
                .mapNotNull { getResource(it.id) }
                .forEach { discretes!!.put(it.id, it) }
    }

    private fun findAnalogs(equips: Collection<MapObject>) {
        for (obj in equips) {
            val measurements = measureProvider?.getAnalogsOfEquip(obj.id, dataSource) ?: continue
            equipMeasurements!!.put(obj.id, measurements)
            measurements.mapNotNull { getResource(it.id) }
                    .forEach { analogs!!.put(it.id, it) }
        }
    }

    // ---------------------------- 拓扑分析开始 -------------------------------

    /**
     * 根据开关状态分析网络拓扑
     */
    fun buildDynamicTopo() : List<Subgraph<TN, Edge, UndirectedGraph<TN, Edge>>> {
        var size = 0
        activeSubgraphes.forEach { size += it.vertexSet().size }
        val cnToTn: MutableMap<String, TN> = HashMap(size)
        // 将闭合开关连在一起的CN都合并在一个TN里
        getAllSwitch().filter { measureProvider!!.getSwitchStatus(it.id, dataSource) }.forEach {
            val terminals = getTerminalsOfEquip(it)
            val cn1 = terminals[0].getProperty("CONNECTIVITYNODE")
            val cn2 = terminals[1].getProperty("CONNECTIVITYNODE")
            @Suppress("LoopToCallChain")
            for(g in activeSubgraphes) {
                if(g.containsEdge(cn1, cn2)) {
                    val tn : TN
                    if(cnToTn.containsKey(cn1)) {
                        if(cnToTn.containsKey(cn2))
                            break
                        tn = cnToTn[cn1]!!
                        tn.cns.add(cn2)
                        cnToTn[cn2] = tn
                    } else if(cnToTn.containsKey(cn2)) {
                        if (cnToTn.containsKey(cn1))
                            break
                        tn = cnToTn[cn2]!!
                        tn.cns.add(cn1)
                        cnToTn[cn1] = tn
                    } else {
                        tn = TN()
                        cnToTn[cn1] = tn
                        cnToTn[cn2] = tn
                        tn.cns.add(cn1)
                        tn.cns.add(cn2)
                    }
                    break
                }
            }
        }
        val tnToGraph = HashMap<TN, UndirectedGraph<TN, Edge>>()
        val islands = LinkedList<Subgraph<TN, Edge, UndirectedGraph<TN, Edge>>>()
        // 开始构建TN与SimpleGraph之间的关系
        for(g in activeSubgraphes) {
            val largeG = SimpleGraph<TN, Edge>(Edge::class.java)
            g.vertexSet().forEach {
                if(!cnToTn.containsKey(it)) {
                    val tn = TN()
                    cnToTn[it] = tn
                    tn.cns.add(it)
                    largeG.addVertex(tn)
                    tnToGraph[tn] = largeG
                } else {
                    val tn = cnToTn[it]!!
                    if(!tnToGraph.containsKey(tn)) {
                        largeG.addVertex(tn)
                        tnToGraph[tn] = largeG
                    }
                }
            }
            g.edgeSet().forEach {
                //如果支路是断开的开关则不加入到图
                if(it.equips.size == 1 && measureProvider!!.getSwitchStatus(it.equips[0], dataSource))
                    return@forEach
                val cn1 = g.getEdgeSource(it)
                val cn2 = g.getEdgeTarget(it)
                val tn1 = cnToTn[cn1]
                val tn2 = cnToTn[cn2]
                if(tn1 != tn2) {
                    val e = largeG.getEdge(tn1, tn2)
                    if (e == null) {//如果不存在边
                        val edge = Edge()
                        edge.equips.addAll(it.equips)
                        largeG.addEdge(tn1, tn2, edge)
                    } else //两条Edge合并
                        e.equips.addAll(it.equips)
                }
            }
            // 连通性分析
            val inspector = ConnectivityInspector(largeG)
            val graphs = inspector.connectedSets() //获得连通子图
            // 开始建立
            graphs.mapTo(islands) { Subgraph<TN, Edge, UndirectedGraph<TN, Edge>>(largeG, it) }
        }
        return islands
    }

    inner class CimXmlHandler : DefaultHandler() {
        // 使用栈这个数据结构来保存
        private val stack = Stack<String>()

        private var obj: MapObject? = null
        private var obj_constructing = false
        private var obj_type: String? = null
        private var startTime : Long = 0

        @Throws(SAXException::class)
        override fun startDocument() {
            println("start document -> parse begin")
            startTime = System.currentTimeMillis()
        }

        @Throws(SAXException::class)
        override fun endDocument() {
            println("end document -> parse finished, time used ${System.currentTimeMillis() - startTime} ms.")
            // 文档结束之后建立对象之间的关系
            endAddPSR()
        }

        @Throws(SAXException::class)
        override fun startElement(uri: String?, localName: String?, qName: String?,
                                  attributes: Attributes?) {
            // 将标签名压入栈
            stack.push(qName)
            // 处理属性
            if (attributes!!.length >= 1) {
                if (attributes.getQName(0) == "rdf:ID") {//判断是否遇见带有id的元素，即二级元素
                    obj = MapObject()    //生成一个新的obj空间
                    obj_constructing = true  //构造开始
                    val index = qName!!.indexOf(":")
                    //记录当前obj_type，用以确认obj构造结束时刻,去掉前缀
                    obj_type = qName.substring(index + 1).toUpperCase()
                    obj!!.id = attributes.getValue(0)    //obj初始化
                    obj!!.type = obj_type
                }

                if (attributes.getQName(0) == "rdf:resource") {
                    var tag: String = qName!!         // 取出标签名
                    if (tag.indexOf('.') >= 0) {//tag的简单处理
                        tag = tag.substring(tag.indexOf('.') + 1, tag.length) //取出原tag中的.后子字符串
                    }
                    var value = attributes.getValue(0)
                    value = value.substring(1, value.length)    //去除#后的sub值,end值需要大1
                    if (value.trim { it <= ' ' } != "") {
                        obj!!.setProperty(tag.toUpperCase(), value)//存入characters
                    }
                }
            }
        }

        @Throws(SAXException::class)
        override fun characters(ch: CharArray?, start: Int, length: Int) {
            val value = String(ch!!, start, length)
            // 取出标签名
            var tag = stack.peek()
            //空白值不保存
            if ((value.trim { it <= ' ' } != "") and (tag != obj_type)) {
                if (obj_constructing) {
                    if (tag.indexOf('.') >= 0) {
                        //取出原tag中的.后子字符串，end值需要大1
                        tag = tag.substring(tag.indexOf('.') + 1, tag.length)
                    }
                    obj!!.setProperty(tag.toUpperCase(), value)//存入characters
                }
            }
        }

        @Throws(SAXException::class)
        override fun endElement(uri: String?, localName: String?, qName: String?) {
            //表示该元素解析完毕，需要从栈中弹出标签
            stack.pop()
            val index = qName!!.indexOf(":")
            if (obj_type == qName.substring(index + 1).toUpperCase()) {//判断obj构造是否结束
                addPSR(obj!!)
                obj_constructing = false//构造结束
            }
        }
    }
}